import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { copyFileSync, mkdirSync, existsSync } from 'fs'
import { resolve } from 'path'

export default defineConfig({
  plugins: [
    react(),
    {
      name: 'copy-assets',
      writeBundle() {
        const assetsDir = resolve(__dirname, 'dist')
        const spritesTarget = resolve(assetsDir, 'sprites')
        const soundsTarget = resolve(assetsDir, 'sounds')
        const cssTarget = resolve(assetsDir, 'css')
        const jsTarget = resolve(assetsDir, 'js')

        if (!existsSync(spritesTarget)) mkdirSync(spritesTarget, { recursive: true })
        if (!existsSync(soundsTarget)) mkdirSync(soundsTarget, { recursive: true })
        if (!existsSync(cssTarget)) mkdirSync(cssTarget, { recursive: true })
        if (!existsSync(jsTarget)) mkdirSync(jsTarget, { recursive: true })

        const copyDir = (src, dest) => {
          const fs = require('fs')
          const path = require('path')
          if (!fs.existsSync(dest)) fs.mkdirSync(dest, { recursive: true })
          const files = fs.readdirSync(src)
          files.forEach(file => {
            const srcPath = path.join(src, file)
            const destPath = path.join(dest, file)
            const stat = fs.statSync(srcPath)
            if (stat.isDirectory()) {
              copyDir(srcPath, destPath)
            } else {
              fs.copyFileSync(srcPath, destPath)
            }
          })
        }

        copyDir(resolve(__dirname, 'sprites'), spritesTarget)
        copyDir(resolve(__dirname, 'sounds'), soundsTarget)
        copyDir(resolve(__dirname, 'css'), cssTarget)
        copyDir(resolve(__dirname, 'js'), jsTarget)
      }
    }
  ],
  root: '.',
  publicDir: 'public',
})
