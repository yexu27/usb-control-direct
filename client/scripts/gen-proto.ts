import { execSync } from 'child_process'
import { resolve } from 'path'
import { mkdirSync } from 'fs'

const root = resolve(__dirname, '..')
const protoFile = resolve(root, 'proto/usb_control.proto')
const outDir = resolve(root, 'src/shared/proto')
const outJs = resolve(outDir, 'usb_control.js')
const outDts = resolve(outDir, 'usb_control.d.ts')

mkdirSync(outDir, { recursive: true })

execSync(
  `npx pbjs -t static-module -w es6 --no-create --no-verify -o "${outJs}" "${protoFile}"`,
  { stdio: 'inherit', cwd: root },
)

execSync(`npx pbts -o "${outDts}" "${outJs}"`, {
  stdio: 'inherit',
  cwd: root,
})

console.log('Proto generation complete.')
