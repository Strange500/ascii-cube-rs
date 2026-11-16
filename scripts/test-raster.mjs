import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
const cubeModule = new URL(
  process.env.CUBE_MODULE || '../pkg/cube.js',
  import.meta.url
)
const { default: init, Cube } = await import(cubeModule.href)
const { memory } = await init({
  module_or_path: await readFile(new URL('cube_bg.wasm', cubeModule))
})
const cube = new Cube(120, 60)
cube.set_zoom(1500)
cube.set_rotation_speed(0, 0, 0)
let holes = 0,
  affected = 0
for (let frame = 0; frame < 180; frame++) {
  cube.set_rotation(frame * 0.037, frame * 0.021, frame * 0.013)
  cube.next_frame()
  const chars = new Uint8Array(memory.buffer, cube.chars_ptr(), 7200)
  let frameHoles = 0
  for (let y = 0; y < 60; y++) {
    const row = chars.subarray(y * 120, (y + 1) * 120)
    const first = row.findIndex(c => c !== 32)
    const last = row.findLastIndex(c => c !== 32)
    for (let x = first; x < last; x++) if (row[x] === 32) frameHoles++
  }
  if (frameHoles) affected++
  holes += frameHoles
}
// Check row-major texture mapping in all four quadrants (static/video use the
// same update_face_fast API). Freeze rotation to make the probes deterministic.
cube.set_rotation(0, 0, 0)
const texture = new Uint8Array(10000)
const colors = new Uint32Array(10000)
const palette = [0xff0000, 0x00ff00, 0x0000ff, 0xffff00]
for (let y = 0; y < 100; y++) {
  for (let x = 0; x < 100; x++) {
    const quadrant = (y >= 50 ? 2 : 0) + (x >= 50 ? 1 : 0)
    texture[y * 100 + x] = 65 + quadrant
    colors[y * 100 + x] = palette[quadrant]
  }
}
cube.update_face_fast(5, texture, colors)
cube.next_frame()
const charFrame = new Uint8Array(memory.buffer, cube.chars_ptr(), 7200)
const colorFrame = new Uint32Array(memory.buffer, cube.colors_ptr(), 7200)
for (const [quadrant, x, y] of [
  [0, 45, 22],
  [1, 75, 22],
  [2, 45, 38],
  [3, 75, 38]
]) {
  assert.equal(
    charFrame[y * 120 + x],
    65 + quadrant,
    'Texture coordinates must not be transposed'
  )
  assert.equal(
    colorFrame[y * 120 + x],
    palette[quadrant],
    'Every texture quadrant must receive its color'
  )
}
const stillFrame = charFrame.slice()
cube.next_frame()
assert.deepEqual(
  new Uint8Array(memory.buffer, cube.chars_ptr(), 7200),
  stillFrame,
  'A frozen cube must render identically'
)
texture.fill(88)
colors.fill(0xabcdef)
cube.update_face_fast(5, texture, colors)
cube.next_frame()
assert.equal(
  new Uint8Array(memory.buffer, cube.chars_ptr(), 7200)[30 * 120 + 60],
  88,
  'Animated texture updates must reach the current frame'
)
cube.free()
console.log(
  '100x100 texture mapping, colors, frozen frames and live updates: OK'
)
console.log(JSON.stringify({ frames: 180, affected, holes }))
assert.equal(
  holes,
  0,
  'A solid convex cube must not have empty cells inside a scanline'
)

// The persistent scene uses a larger, uncropped grid and a higher projection zoom.
// Raising zoom past the upstream sampler's useful density creates interior holes.
const sceneCube = new Cube(200, 100)
sceneCube.set_zoom(3300)
sceneCube.set_rotation_speed(0, 0, 0)
let sceneHoles = 0
for (let frame = 0; frame < 720; frame++) {
  sceneCube.set_rotation(frame * 0.037, frame * 0.053, frame * 0.019)
  sceneCube.next_frame()
  const chars = new Uint8Array(memory.buffer, sceneCube.chars_ptr(), 200 * 100)
  for (let y = 0; y < 100; y++) {
    const row = chars.subarray(y * 200, (y + 1) * 200)
    const first = row.findIndex(c => c !== 32)
    const last = row.findLastIndex(c => c !== 32)
    for (let x = first; x < last; x++) if (row[x] === 32) sceneHoles++
  }
}
// Higher sampling must not change the legacy 100×100 upload coordinates.
sceneCube.set_rotation(0, 0, 0)
for (let y = 0; y < 100; y++) for (let x = 0; x < 100; x++) {
  const q = (y >= 50 ? 2 : 0) + (x >= 50 ? 1 : 0)
  texture[y * 100 + x] = 65 + q
  colors[y * 100 + x] = palette[q]
}
sceneCube.update_face_fast(5, texture, colors)
sceneCube.next_frame()
const sceneChars = new Uint8Array(memory.buffer, sceneCube.chars_ptr(), 20000)
const sceneColors = new Uint32Array(memory.buffer, sceneCube.colors_ptr(), 20000)
for (const [q, x, y] of [[0,80,40], [1,120,40], [2,80,60], [3,120,60]]) {
  assert.equal(sceneChars[y * 200 + x], 65 + q, 'High-resolution texture quadrant')
  assert.equal(sceneColors[y * 200 + x], palette[q], 'High-resolution texture color')
}
sceneCube.free()
assert.equal(
  sceneHoles,
  0,
  'Scene projection must stay watertight at 200×100 / zoom 3300'
)
console.log(JSON.stringify({ sceneFrames: 720, sceneHoles }))
