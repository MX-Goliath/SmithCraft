# Сборка и дистрибутивы SmithCraft

Поддерживаемые пакеты: Windows x86-64 (установщик и portable ZIP), Linux x86-64
(AppImage). Версия берётся из `Cargo.toml`. Rust 1.99.1 закреплён в
`rust-toolchain.toml`, зависимости — в `Cargo.lock`.

## Windows

Нужны Rust и один из компиляторов:

- MSVC: Visual Studio Build Tools с компонентом Desktop development with C++.
- GNU: Rust `x86_64-pc-windows-gnu` и MinGW-w64; локальный w64devkit в
  `.tools/w64devkit/bin` скрипт находит автоматически.

Для установщика дополнительно нужен [Inno Setup 6](https://jrsoftware.org/isinfo.php).
`tools/install-inno.ps1` скачивает версию 6.7.4 с проверкой SHA-256 и устанавливает
компилятор в `.tools/inno` без прав администратора.
Он устанавливается только на машине сборки. Целевой компьютер не требует Rust,
Visual Studio, Inno Setup или отдельной установки CRT. Для MSVC включён статический CRT.

```powershell
# Портативный ZIP, с проверкой тестов. По умолчанию используется host target.
.\tools\package-windows.ps1 -Test

# Установщик и портативный ZIP (как в CI).
rustup target add x86_64-pc-windows-msvc
.\tools\install-inno.ps1
.\tools\package-windows.ps1 -Target x86_64-pc-windows-msvc -Installer -Test

# Явный путь к компилятору Inno Setup, если его нет в стандартном каталоге.
.\tools\package-windows.ps1 -Installer -Iscc 'C:\Program Files (x86)\Inno Setup 6\ISCC.exe'
```

Результаты в `dist/artifacts/`:

```text
SmithCraft-<version>-windows-x86_64-portable.zip
SmithCraft-<version>-windows-x86_64-setup.exe
<имя каждого пакета>.sha256
```

Распакуйте ZIP целиком в доступную для записи папку и запустите `SmithCraft.exe`.
Маркер `portable.txt` включает хранение настроек в `data/state.ron` рядом с exe.
Удаление маркера возвращает стандартное хранение настроек в профиле пользователя.
Пакет не содержит настройки и историю компьютера сборки.
Установщик работает без запроса прав администратора, создаёт ярлык в меню «Пуск»
и устанавливает программу в `%LOCALAPPDATA%\Programs\SmithCraft`.
Удаление через «Приложения» сохраняет пользовательские проекты и настройки.

Старый `build.ps1 -Test -Release` по-прежнему создаёт `dist/SmithCraft.exe`
для локальной разработки; он не заменяет упаковку дистрибутивов.

## Linux / AppImage

Собирайте **на Linux x86-64**. Эталонная среда CI — Ubuntu 22.04 (glibc 2.35).
Для совместимости с этой базой не собирайте релиз на более новом дистрибутиве.
На Windows используйте Linux-машину, WSL2 с Ubuntu 22.04 или GitHub Actions.
Скрипт Windows не выдаёт Windows exe за Linux AppImage.

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config curl python3 file \
  libx11-dev libxi-dev libxcursor-dev libxrandr-dev libxcb-shape0-dev \
  libxcb-xfixes0-dev libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev \
  libgl1-mesa-dev libegl1-mesa-dev libgl1-mesa-dri libfuse2 \
  librsvg2-bin desktop-file-utils xvfb xauth dbus-x11

cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
bash tools/package-linux.sh
bash tools/smoke-linux.sh dist/artifacts/*.AppImage
```

Результат: `dist/artifacts/SmithCraft-<version>-linux-x86_64.AppImage` и
контрольная сумма `.sha256`. Пакет содержит бинарник, библиотеки X11/Wayland,
значок, desktop-файл, примеры и исходник справки. Поддержка X11 и Wayland
включена в eframe. Драйвер OpenGL и glibc предоставляет система пользователя.
Системный D-Bus и XDG Desktop Portal с подходящим desktop backend используются
для открытия/сохранения файлов (например, `xdg-desktop-portal-gtk` на X11).
Нативная печать Windows, Windows-диалог выбора цвета и просмотрщик CHM на Linux недоступны; экспорт и
предпросмотр работают, текст справки включён в пакет.

```bash
chmod +x SmithCraft-<version>-linux-x86_64.AppImage
./SmithCraft-<version>-linux-x86_64.AppImage
./SmithCraft-<version>-linux-x86_64.AppImage --demo

# Запуск на системах без FUSE:
APPIMAGE_EXTRACT_AND_RUN=1 ./SmithCraft-<version>-linux-x86_64.AppImage
```

Инструменты упаковки скачиваются в `.tools/appimage/` из фиксированных релизов:
linuxdeploy `1-alpha-20251107-1`, appimagetool `1.9.1`, type2-runtime `20251108`.
Перед выполнением проверяется закреплённая SHA-256 каждого файла. Для обновления
инструментов измените версию и сумму в `tools/package-linux.sh` одновременно.
Во время сборки требуется интернет; готовая программа работает офлайн.
Скрипт принимает `--skip-build` для упаковки уже собранного release-бинарника.

## Автоматическая сборка

`.github/workflows/build.yml` запускается для push в `main`/`master`, тегов `v*`,
pull request и вручную через **Actions → Build distributions → Run workflow**.
Windows и Linux собираются независимо. Оба задания запускают форматирование,
тесты и Clippy. Windows проверяет portable exe, установку и удаление setup; Linux
запускает упакованный AppImage в Xvfb с программным OpenGL и чистым профилем.
Проверка запуска GUI в CI выполняется для X11; Wayland требует отдельной
ручной проверки в соответствующей desktop-сессии.

Скачайте результаты в разделе **Artifacts** завершившегося workflow:
`SmithCraft-windows-x86_64` и `SmithCraft-linux-x86_64`.
Workflow не публикует релиз автоматически. Перед релизом проверьте пакеты на
чистой целевой системе и сверяйте версию тега с `Cargo.toml`.

Документация инструментов:
[linuxdeploy](https://github.com/linuxdeploy/linuxdeploy),
[appimagetool](https://github.com/AppImage/appimagetool),
[Inno Setup](https://jrsoftware.org/ishelp/).
