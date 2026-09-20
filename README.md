# sMusic (Tauri)

Локальный плеер под FLAC/WAV/MP3, замена AppleMusic + отдельного `applemusic-discord-rpc` моста. На Tauri (Rust + голый HTML/CSS/JS, без фреймворков и билд-шага), чтобы не жрать проц как Electron.

Фичи:
- играет свою музыку (сам закидываешь папку с файлами), без стриминга и скачивания
- реальное качество звука — без пересжатия/downsampling
- Discord Rich Presence: "слушает", название трека без автора, обложка, прогресс-бар
- сворачивается в трей
- дизайн в стиле btwincel.pw — чёрный минимализм, JetBrains Mono, зерно, анимации

## Разработка

```
npm install
npm run tauri dev
```

## Структура

- `src/` — фронт, голый JS без сборки
- `src-tauri/` — бэкенд на Rust: аудио-движок (rodio), теги (lofty), трей, Discord RPC
- `cover-relay/` — маленький FastAPI-сервис для обложек в Discord (деплоится отдельно на финка2)
