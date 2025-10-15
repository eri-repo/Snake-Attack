import React from 'react'
import { createRoot } from 'react-dom/client'
import App from './App'
import './global.css'

console.log('main.jsx: booting React entry')

createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
)
