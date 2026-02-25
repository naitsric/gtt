# gtt — Git Time Tracker

> Estima horas trabajadas y montos a facturar directamente desde tu historial de commits.

```
$ gtt report --client "Startup X" --last-month

Cliente: Startup X
Periodo: 01/01/2026 — 31/01/2026

+-----------+----------+--------+---------+------------------------------+
| Fecha     | Sesiones | Horas  | Commits | Repos                        |
+-----------+----------+--------+---------+------------------------------+
| Lun 05/01 |        2 | 3h 15m |       5 | startupx-web                 |
| Mar 06/01 |        1 | 1h 40m |       3 | startupx-api                 |
| Mie 07/01 |        3 | 4h 50m |       8 | startupx-web, startupx-api   |
| Vie 09/01 |        1 | 2h 10m |       4 | startupx-web                 |
| Total     |        7 | 11h 55m|      20 |                              |
+-----------+----------+--------+---------+------------------------------+

Monto: 11.92h × 80/h = 953.33 USD
```

---

## El problema

Los freelancers que cobran por hora pierden dinero constantemente porque reconstruir el tiempo trabajado a mano es doloroso. Los timers manuales se olvidan. Las estimaciones a ojo subestiman.

**Lo que ya tienes:** cada commit tiene un timestamp exacto. `gtt` los analiza para detectar sesiones de trabajo y calcular las horas reales.

**Por qué las alternativas no cierran el loop:**

| Herramienta | Problema |
|---|---|
| `git-hours` | Sin clientes ni tasas. Abandonado (roto en Node 18+). |
| GTM | Requiere plugins de editor. Pierde datos con rebase/squash. |
| WakaTime | $8/mes, envía código a servidores externos. |
| Toggl / Clockify | Requieren timers manuales — el problema original. |

`gtt` es la única herramienta que va de **repos → sesiones → horas → monto a facturar** en un solo comando, sin servidores ni suscripciones.

---

## Instalación

### Desde binario (recomendado)

Descarga el binario para tu plataforma desde la [página de releases](https://github.com/tu-usuario/gtt/releases):

```bash
# Linux / macOS
curl -L https://github.com/tu-usuario/gtt/releases/latest/download/gtt-linux-x86_64 -o gtt
chmod +x gtt
sudo mv gtt /usr/local/bin/
```

### Con Cargo

```bash
cargo install gtt
```

### Desde fuente

```bash
git clone https://github.com/tu-usuario/gtt
cd gtt
cargo build --release
# El binario queda en target/release/gtt
```

---

## Inicio rápido

**1. Configurar clientes y repos:**

```bash
gtt init
```

El wizard te pregunta por tus clientes, las rutas a sus repositorios y tu tasa horaria. Genera `~/.config/gtt/config.toml` automáticamente.

**2. Ver horas de hoy y esta semana:**

```bash
gtt status
```

**3. Ver el reporte del mes pasado:**

```bash
gtt report --client "Startup X" --last-month
```

**4. Verificar sesiones antes de facturar:**

```bash
gtt verify --client "Startup X" --last-month
```

**5. Exportar a CSV para tu sistema de facturación:**

```bash
gtt export --client "Startup X" --last-month --format csv
# Genera: gtt-startup-x-2026-01.csv
```

---

## Configuración

El archivo de configuración vive en `~/.config/gtt/config.toml`:

```toml
[client."Startup X"]
repos = [
    "/home/user/startupx-web",
    "/home/user/startupx-api",
]
hourly_rate = 80
currency = "USD"

[client."Agencia Y"]
repos = ["/home/user/agencia-landing"]
hourly_rate = 60
currency = "EUR"

[settings]
session_gap_minutes = 120   # inactividad > 2h = nueva sesión
first_commit_minutes = 30   # tiempo base por primer commit de sesión
exclude_weekends = false     # no cruzar sesiones en fin de semana
```

### Opciones de `[settings]`

| Opción | Default | Descripción |
|---|---|---|
| `session_gap_minutes` | `120` | Minutos de inactividad que inician una nueva sesión |
| `first_commit_minutes` | `30` | Minutos base asignados al primer commit de cada sesión |
| `exclude_weekends` | `false` | Evita cruzar sesiones entre viernes y lunes |
| `bot_authors` | `["dependabot[bot]", ...]` | Autores excluidos del análisis |

Para editar el config directamente:

```bash
gtt config show    # ver configuración actual
gtt config edit    # abrir en $EDITOR
```

---

## Comandos

### `gtt init`

Setup interactivo. Crea o sobreescribe `~/.config/gtt/config.toml`.

```bash
gtt init
```

---

### `gtt status`

Resumen rápido de horas de hoy y esta semana por cliente.

```bash
gtt status

# gtt status
#   Hoy: 25/02/2026    Esta semana: 23/02/2026 — 25/02/2026
#
#   Startup X — Hoy: 2h 30m  (4 commits)   Esta semana: 8h 15m  (14 commits)
#   Agencia Y — Hoy: 0m  (0 commits)        Esta semana: 3h 40m  (7 commits)
```

---

### `gtt report`

Reporte de horas por cliente. Soporta múltiples formatos y rangos de fecha.

```bash
# Mes pasado (más común para facturar)
gtt report --client "Startup X" --last-month

# Semana pasada
gtt report --client "Startup X" --last-week

# Rango personalizado
gtt report --client "Startup X" --since 2026-01-01 --until 2026-01-31

# Todos los clientes
gtt report --last-month

# Formato CSV (imprime a stdout)
gtt report --client "Startup X" --last-month --format csv

# Formato JSON
gtt report --client "Startup X" --last-month --format json

# Guardar en archivo específico
gtt report --client "Startup X" --last-month --format csv --output enero-2026.csv
```

**Flags disponibles:**

| Flag | Descripción |
|---|---|
| `--client <nombre>` | Filtrar por cliente. Sin flag, reporta todos. |
| `--last-week` | Semana pasada (lunes a domingo) |
| `--last-month` | Mes calendario anterior |
| `--since <YYYY-MM-DD>` | Inicio del rango |
| `--until <YYYY-MM-DD>` | Fin del rango |
| `--format <fmt>` | `table` (default), `csv`, `json` |
| `--output <archivo>` | Guardar en archivo en vez de stdout |

---

### `gtt verify`

Lista las sesiones detectadas con timestamps y commits incluidos. Úsalo para validar que el análisis coincide con tu percepción antes de facturar.

```bash
gtt verify --client "Startup X" --last-month

# Verificar sesiones: Startup X
# Periodo: 01/01/2026 — 31/01/2026
#
# ── Monday 05/01/2026 (2 sesiones, 3h 15m) ──
#   Sesión 1:  09:15 → 10:45  (1h 30m, 3 commits)
#     09:15 a3f2e1b feat: add user authentication
#     09:52 b1c4d5e fix: handle invalid tokens
#     10:45 c2d3e4f test: authentication edge cases
#
#   Sesión 2:  15:30 → 17:00  (1h 45m, 2 commits)
#     15:30 d4e5f6a refactor: extract auth service
#     17:00 e5f6a7b docs: update API documentation
```

**Flags:** mismos que `gtt report` (excepto `--format` y `--output`).

---

### `gtt export`

Alias de `report` con generación automática de nombre de archivo.

```bash
# Genera gtt-startup-x-2026-01.csv en el directorio actual
gtt export --client "Startup X" --last-month --format csv

# JSON
gtt export --client "Startup X" --last-month --format json

# Nombre personalizado
gtt export --client "Startup X" --last-month --format csv --output factura-enero.csv
```

El nombre generado sigue el patrón `gtt-<cliente>-<YYYY-MM>.<formato>`.

---

### `gtt config`

```bash
gtt config show   # imprime el config actual
gtt config edit   # abre en $EDITOR (o nano si no está definido)
```

---

## Cómo funciona el algoritmo de sesiones

`gtt` analiza el historial de commits para inferir cuándo trabajaste:

1. **Ordena** todos los commits por `author date` (no commit date — robusto ante `git rebase` y `git commit --amend`).

2. **Detecta sesiones** comparando pares consecutivos de commits:
   - Si el gap es **> `session_gap_minutes`** (default: 2 horas) → nueva sesión
   - Si el par **cruza medianoche** → nueva sesión (aunque el gap sea menor)
   - En caso contrario → mismo bloque de trabajo, el gap cuenta como tiempo trabajado

3. **Agrega tiempo base** al primer commit de cada sesión (`first_commit_minutes`, default: 30 min), para compensar el tiempo previo al primer commit.

4. **Excluye bots**: commits de Dependabot, GitHub Actions y similares se ignoran por defecto.

```
Commits:  09:00  09:45  10:30        15:00  15:20
          |------|------|             |------|
          45min  45min               20min
          ←   sesión 1   →           ← sesión 2 →

Sesión 1: 30min base + 45 + 45 = 2h 0m
Sesión 2: 30min base + 20 = 50m
Total:    2h 50m
```

> **Nota:** `gtt` produce **estimaciones**, no registros exactos. Usa `gtt verify` para revisar las sesiones detectadas antes de facturar. El README de cada reporte sugiere revisarlo con el cliente si hay disputas.

---

## Formatos de exportación

### CSV

Compatible con FreshBooks, Wave, Invoice Ninja y cualquier hoja de cálculo.

```csv
date,sessions,hours,minutes,commits,repos,amount,currency
2026-01-05,2,3.2500,195,5,startupx-web,260.00,USD
2026-01-06,1,1.6667,100,3,startupx-api,133.33,USD
```

### JSON

```json
{
  "client": "Startup X",
  "period_start": "2026-01-01",
  "period_end": "2026-01-31",
  "total_minutes": 715,
  "total_hours": 11.92,
  "total_commits": 20,
  "hourly_rate": 80.0,
  "currency": "USD",
  "billable_amount": 953.33,
  "days": [
    {
      "date": "2026-01-05",
      "sessions": 2,
      "total_minutes": 195,
      "total_hours": 3.25,
      "total_commits": 5,
      "repos": ["startupx-web"],
      "amount": 260.0
    }
  ]
}
```

---

## Casos de uso avanzados

### Repos compartidos con otros devs

`gtt` filtra automáticamente por el email configurado en `git config user.email` de cada repositorio. Solo tus commits cuentan.

### Múltiples repos por cliente

```toml
[client."Startup X"]
repos = [
    "/home/user/startupx-web",
    "/home/user/startupx-api",
    "/home/user/startupx-mobile",
]
hourly_rate = 80
currency = "USD"
```

Los commits de todos los repos se combinan y las sesiones se detectan entre ellos. Un commit a `api` y un commit a `web` con 40 minutos de diferencia son parte de la misma sesión.

### Workflows de facturación

```bash
# 1. Revisar sesiones detectadas
gtt verify --client "Startup X" --last-month

# 2. Si todo se ve bien, generar reporte
gtt report --client "Startup X" --last-month

# 3. Exportar para importar a tu sistema de facturación
gtt export --client "Startup X" --last-month --format csv
```

---

## Preguntas frecuentes

**¿Qué pasa si hice `git rebase` o `git commit --amend`?**
`gtt` siempre usa el `author date` (la fecha original del commit), no el `commit date` (que cambia con rebase/amend). Tus horas son robustas ante cualquier reescritura del historial.

**¿Por qué no aparecen mis commits?**
`gtt` filtra por el email del autor configurado en `git config user.email`. Verifica que el email en tu repo coincide con el autor de los commits: `git log --format="%ae" | head -5`.

**¿Mis commits de bots inflan el tiempo?**
No. Los commits de Dependabot, GitHub Actions y cualquier autor que termine en `[bot]` se excluyen automáticamente. Puedes agregar más exclusiones en `bot_authors` del config.

**¿Es exacto?**
Es una estimación. El algoritmo no puede saber cuánto tiempo pasaste pensando antes del primer commit o revisando código sin commitear. Por eso el parámetro `first_commit_minutes` existe — y por eso `gtt verify` te permite revisar las sesiones antes de facturar.

**¿Mis datos salen de mi máquina?**
Nunca. `gtt` es un binario que corre completamente local. No tiene servidores, no tiene telemetría, no hay red.

---

## Contribuir

```bash
git clone https://github.com/tu-usuario/gtt
cd gtt
cargo test        # correr todos los tests
cargo check       # verificar sin compilar
cargo build       # build de desarrollo
```

Los tests del algoritmo de sesiones están en `tests/session_algorithm_test.rs`. Si modificas `src/session/analyzer.rs`, agrega tests para los casos edge que estás cubriendo.

---

## Licencia

MIT — úsalo, modifícalo, distribúyelo.
