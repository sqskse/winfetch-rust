# winfetch-rust

[![Rust CI](https://github.com/sqskse/winfetch-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/sqskse/winfetch-rust/actions/workflows/ci.yml)

Утилита для быстрой и подробной инвентаризации системы Windows. Написана на Rust.

## Возможности
- **Железо и ОС**: Подробная информация об ОС, пользователе, CPU, RAM, дисках и сети (через `sysinfo`).
- **Глубокий аудит**: Видеокарты, материнская плата, BIOS, батарея (через WMI).
- **Службы Windows**: Вывод запущенных и автоматических служб (`Win32_Service`).
- **Установленный софт**: Список установленных программ (чтение реестра, без вызовов медленного `Win32_Product`).
- **Экспорт данных**: Вывод в консоль, JSON для скриптов или сохранение в файл (`.txt` / `.json`).
- **Не требует прав администратора** (кроме случаев, когда WMI ограничен групповыми политиками).

## Сборка и установка
Для сборки из исходников вам понадобится установленный Rust (MSVC toolchain):

```powershell
git clone https://github.com/sqskse/winfetch-rust.git
cd winfetch-rust
cargo build --release
```

## Использование

```powershell
# Полный отчет в консоль
winfetch
```

# Вывод в формате JSON
```
winfetch --json
```

# Только конкретные секции
```
winfetch --section cpu,memory,gpu,services
```

# Сохранение отчета в файл
```
winfetch -o my_pc.txt
```
```
winfetch -o my_pc.json
```
