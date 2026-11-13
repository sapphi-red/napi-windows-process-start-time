import type { UserConfig } from 'tsdown'

const config: UserConfig = {
  entry: 'index.ts',
  format: 'esm',
  platform: 'node',
  target: 'node20',
  outDir: 'dist',
  fixedExtension: false,
  dts: true,
  onSuccess: 'node scripts/build.ts',
}

export default config
