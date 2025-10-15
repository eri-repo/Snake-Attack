# Snakie (wrapped with Vite + React)

This project wraps the legacy SnakeAttack game with a minimal React + Vite dev setup so you can run `npm run dev` and see the game in the browser.

Quick start:

1. Install dependencies

```bash
cd /home/dean/Current_Projects/SnakeGame
npm install
```

2. Start dev server

```bash
npm run dev
```

3. Open the printed URL (typically `http://localhost:5173`) in your browser.

Notes:
- The legacy game scripts and assets remain under `js/`, `sprites/`, `css/`, and `sounds/`. The Vite dev server serves files from `public/` and the project root.
- React is used only as a small wrapper to mount the game's `canvas` and dynamically load the game's original scripts in order so the legacy globals (`CMain`, etc.) work unchanged.
- If you want the app bundled for production, run `npm run build` and then `npm run preview` to test the built output.
