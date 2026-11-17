# lekalo-php-laravel

Framework-пакет Laravel для [Lekalo](https://github.com/ichinya/lekalo), подключаемый внутри процесса языкового адаптера [lekalo-php](https://github.com/ichinya/lekalo-php) через versioned provider API.

- Laravel-факты и сопоставления с семантической моделью: routes, controllers, Policies, FormRequest, storage mappings;
- интеграции Laramago и Laratesto (Mago + Laravel-пресеты, Testo + Laravel-окружение);
- библиотечный пакет с manifest и схемами — собственный демон не требуется.

## Состав (stage 4, M2)

| Путь | Содержимое |
| --- | --- |
| `src/laravel-provider.php` | Дескриптор провайдера `lekalo-php-laravel@0.1.0` на API `lekalo/provider/v0.1.0`: capability matrix по шести семействам (`migrations.emit`, `routes.emit`, `scenario.runners`, `scenario.emit`, `native.gates`, `lint.framework-semantics`) и хуки загрузки модулей. |
| `src/laravel-migrations.php` | Эмиттер миграций (issue #57): контракт `lekalo/laravel-migration-input/v0.4.0`, рендер Illuminate-классов, append-only ledger. |
| `src/laravel-lint.php` | Eloquent-факты lint (базовый класс модели). |
| `src/{scenario-emit,route-emit,native-policy,native-plan}.php` | Эмиттер Laratesto-тестов, Illuminate-роутер, Composer/artisan execution policy и планировщик native-гейтов. |
| `composer-gates-policy.json` | Composer execution policy (custody-байты подтверждений; verbatim встраивается в единый артефакт). |
| `tests/laravel.php` | Сюита пакета: 41 проверка — регистрация в `lekalo-php`, matrix, раннеры, lint-факты, полиси и юнит-ноги эмиттера миграций (перенесены из сюиты протокола монорепы). |
| `lekalo.package.json` | Манифест правил: принадлежащие пакету модули, Composer policy и совместимый provider API. |

## Композиция с PHP runtime

```sh
php -n ../lekalo-php/build.php --rules . \
  --id lekalo-target-php-laravel --target php-laravel \
  --output /tmp/laravel-adapter.php
```

Сборщик принадлежит `lekalo-php`; этот пакет объявляет свои модули и ресурсы
в `lekalo.package.json` (`kind: rules`). Ядро Lekalo не перечисляет файлы
Laravel. Другой набор правил может использовать тот же provider API и свой
checkout. Пересекающиеся capability families двух providers отказывают;
автоматического приоритета по имени пакета нет.

Манифест описывает сборку PHP-артефакта, а не доставку пакетов и не
пользовательскую конфигурацию. Resolver произвольных Git-репозиториев,
версия PHP-интерпретатора и конфигурация компонентов отслеживаются в #239/#241.

## Запуск

Пакет работает на всех актуальных версиях PHP (8.0–8.6; 8.6 — RC до
GA) вместе с языковым адаптером; хост-интерпретатор не нужен.

```sh
php -n tests/laravel.php          # 41 проверка; резолвит ../lekalo-php/src или LEKALO_PHP_SRC
scripts/version-matrix.sh         # сюита на php 8.0…8.6 через docker
```

Общий план отделения — [ichinya/lekalo#241](https://github.com/ichinya/lekalo/issues/241); зависимость на совместимую версию lekalo-php — [ichinya/lekalo#239](https://github.com/ichinya/lekalo/issues/239).

Статус: код перенесён из `adapters/php-laravel` монорепы (PR ichinya/lekalo#243); на переходном этапе монорепа собирает единый `adapter.php` из закреплённых (vendored) выпусков этого пакета и `lekalo-php`.
