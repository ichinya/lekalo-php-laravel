# lekalo-php-laravel

Framework-пакет Laravel для [Lekalo](https://github.com/ichinya/lekalo), подключаемый внутри процесса языкового адаптера [lekalo-php](https://github.com/ichinya/lekalo-php) через versioned provider API.

- Laravel-факты и сопоставления с семантической моделью: routes, controllers, Policies, FormRequest, storage mappings;
- интеграции Laramago и Laratesto (Mago + Laravel-пресеты, Testo + Laravel-окружение);
- библиотечный пакет с manifest и схемами — собственный демон не требуется.

## Переносимые правила

В конфигурации компонента укажите `rules: [gh/ichinya/lekalo-php-laravel]`. Публичный пакет версии 0.2.0 поставляет `contracts/http.json`. Его правило `health-route` требует маршрут `GET /health`. Это явная политика выбранного пакета. Другой runtime может применять те же правила, если он выдаёт факты `http.routes/v1` вида `{"routes":[{"method":"GET","path":"/health"}]}`.

Для PHP пакет описывает класс регистрации и методы в `bindings.php.static-router/v1`. Поддерживаются прямые вызовы с литеральным путём в `routes/web.php` или `web.php` относительно компонента. Группы, префиксы и динамические пути требуют более полного анализатора. Проверка сообщает о неполном анализе при встрече таких конструкций.

## Состав PHP-провайдера

| Путь | Содержимое |
| --- | --- |
| `src/laravel-provider.php` | Дескриптор провайдера `lekalo-php-laravel@0.1.0` на API `lekalo/provider/v0.1.0`: capability matrix по шести семействам (`migrations.emit`, `routes.emit`, `scenario.runners`, `scenario.emit`, `native.gates`, `lint.framework-semantics`) и хуки загрузки модулей. |
| `src/laravel-migrations.php` | Эмиттер миграций (issue #57): контракт `lekalo/laravel-migration-input/v0.4.0`, рендер Illuminate-классов, append-only ledger. |
| `src/laravel-lint.php` | Eloquent-факты lint (базовый класс модели). |
| `src/{scenario-emit,route-emit,native-policy,native-plan}.php` | Эмиттер Laratesto-тестов, Illuminate-роутер, Composer/artisan execution policy и планировщик native-гейтов. |
| `composer-gates-policy.json` | Composer execution policy (custody-байты подтверждений; verbatim встраивается в единый артефакт). |
| `tests/laravel.php` | Сюита пакета: 41 проверка — регистрация в `lekalo-php`, matrix, раннеры, lint-факты, полиси и юнит-ноги эмиттера миграций (перенесены из сюиты протокола монорепы). |
| `lekalo.php-build.json` | Манифест сборки PHP-провайдера: принадлежащие пакету модули, Composer policy и совместимый provider API. |

## Композиция с PHP runtime

```sh
php -n ../lekalo-php/build.php --rules . \
  --id lekalo-target-php-laravel --target php-laravel \
  --output /tmp/laravel-adapter.php
```

Сборщик принадлежит `lekalo-php`; этот пакет объявляет свои модули и ресурсы
в `lekalo.php-build.json` (`kind: rules`). Ядро Lekalo не перечисляет файлы
Laravel. Другой набор правил может использовать тот же provider API и свой
checkout. Пересекающиеся capability families двух providers отказывают;
автоматического приоритета по имени пакета нет.

## Запуск

Пакет работает на всех актуальных версиях PHP (8.0–8.6; 8.6 — RC до
GA) вместе с языковым адаптером; хост-интерпретатор не нужен.

```sh
php -n tests/laravel.php          # 41 проверка; резолвит ../lekalo-php/src или LEKALO_PHP_SRC
scripts/version-matrix.sh         # сюита на php 8.0…8.6 через docker
```

Общий план отделения — [ichinya/lekalo#241](https://github.com/ichinya/lekalo/issues/241); зависимость на совместимую версию lekalo-php — [ichinya/lekalo#239](https://github.com/ichinya/lekalo/issues/239).

В `compat/adapter.php` хранится совместимый артефакт для пользователей target-протокола. Команда сборки выше воспроизводит его байты.

Помимо переносимых правил, пакет поставляет валидатор legacy-входов
`lekalo/operations` и `lekalo/routes`, которые используются до генерации.
Установите его командой `cargo install --locked --path crates/attachments`
и укажите в конфигурации проекта оба сервиса. После этого выполните
`lekalo runtime lock`.

```yaml
services:
  attachments/operations: {command: [lekalo-laravel-attachments]}
  attachments/routes: {command: [lekalo-laravel-attachments]}
```

Процесс получает текст attachment, канонический IR, проверенные привязки
ошибок, транспорт и дайджесты. Возвращаемые findings сохраняют прежние коды.
Для проверки валидатора используйте `cargo test --locked --manifest-path crates/attachments/Cargo.toml`.
