# Server Blocker — архитектура и план MITM-backend

Документ для STALZONE JVM Wrapper. Текущая реализация (v1.5.4): **Windows Firewall** (`STALZONE-SB-*`, порты 29450–29460). **WinDivert на клиенте не используется.**

Рекомендация SilentBless: не строить чистый WinDivert-SBT на клиенте — это конфликтует с ExitLag/GearUP. Вместо этого — серверный MITM и API `address_list` на `backend-*.stalzone`.

---

## 1. Сравнение подходов

| Критерий | Firewall (сейчас) | WinDivert (клиент) | MITM-backend (план) |
|----------|---------------------|--------------------|---------------------|
| Где блокируется | Локально: `New-NetFirewallRule` | Перехват пакетов в ядре | На сервере: прокси/маршрутизация |
| Привилегии | UAC при apply/clear | Драйвер + админ | Минимум на клиенте (HTTPS) |
| ExitLag / GearUP | Обычно не конфликтует | Двойной перехват стека | Клиент не трогает стек |
| Обновление каталога | Bundled JSON + live RU `address_list` | Тот же каталог | Единый `address_list` + routing rules |
| Офлайн | Да, после apply | Да | Нужен fallback (firewall) |

**Вывод:** firewall — дефолт; WinDivert — не рекомендуется; MITM-backend — для пользователей с бустерами.

---

## 2. Текущий код

| Файл | Назначение |
|------|------------|
| `src/server-blocker.js` | UI, каталог, пинг, auto-best |
| `src/server-blocker-logic.js` | blocklist/allowlist, скрытие плохого пинга |
| `src-tauri/src/server_block.rs` | PowerShell firewall |
| `src/data/servers.json` | EU/NA/SEA bundled |
| `src/data/servers-ru.json` | RU bundled + `GET …/address_list` |

Каталог RU: `https://backend.stalcraftx.ru/address_list?login=User`

Обновление bundled JSON: `npm run refresh:servers`  
(`scripts/refresh-server-catalog.mjs` — RU live + EU/NA/SEA из [unofficial-stalzone-api](https://github.com/Art3mLapa/unofficial-stalzone-api) `static/address_list`).

---

## 3. Схема `address_list` v2 (черновик)

Обратно совместима с текущим `pools[]` + `tunnels[]`:

```json
{
  "version": 2,
  "pools": [
    {
      "name": "MSK2",
      "region": "RU",
      "tunnels": [
        {
          "id": "MSK2-1",
          "name": "MSK2-1",
          "address": "95.213.255.12:29450"
        }
      ]
    }
  ],
  "capabilities": {
    "mitm_routing": true
  }
}
```

Правила маршрутизации (новый POST):

```json
{
  "mode": "blocklist",
  "login": "User",
  "resolved": {
    "deny_hosts": ["85.119.149.41"],
    "allow_hosts": ["95.213.255.12"],
    "port_range": [29450, 29460]
  }
}
```

Клиент уже вычисляет `deny_hosts` через `resolveBlockedHosts()` — backend валидирует и применяет.

---

## 4. REST API (черновик)

| Метод | Путь | Описание |
|-------|------|----------|
| `GET` | `/health` | Статус MITM |
| `GET` | `/address_list?login=` | Каталог v2 |
| `POST` | `/routing/rules` | Применить deny/allow |
| `DELETE` | `/routing/rules/{id}` | Снять правила |

---

## 5. Интеграция Tauri (план)

```
enum BlockingBackend { Firewall, MitmBackend, Disabled }
```

- Детекция бустера: процессы `exitlag.exe`, `GearUP.exe`, …
- Бустер + доступный MITM → серверная блокировка без UAC
- Иначе → firewall (как сейчас)
- `stop` снимает и firewall, и MITM-правила

Новые модули (фаза 1+): `mitm_client.rs`, `booster_detect.rs`, `blocking.rs`.

---

## 6. Rollout

| Фаза | Содержание |
|------|------------|
| **0** (v1.5.4) | Предупреждение в UI, этот документ, firewall GA |
| **1** | MVP: `POST /routing/rules`, клиент `backend: mitm` |
| **2** | Персистентность, региональные backend, HMAC |
| **3** | Self-hosted docker-compose для комьюнити |

---

## 7. Риски

| Риск | Митигация |
|------|-----------|
| WinDivert + бустер | Не внедрять WinDivert; предупреждение в UI |
| MITM backend недоступен | Fallback на firewall |
| Устаревшие IP | TTL каталога + «Обновить RU» |

---

## 8. Открытые вопросы

1. Где в цепочке RU/EU стоит MITM — на том же хосте, что `address_list`?
2. Достаточно ли `login=User` или нужен EXBO account ID?
3. TTL правил: 6 ч или 24 ч?

---

*OSS STALZONE JVM Wrapper · Nyrokume, SilentBless*
