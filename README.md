# lekalo-php-laravel

Framework-пакет Laravel для [Lekalo](https://github.com/ichinya/lekalo), подключаемый внутри процесса языкового адаптера [lekalo-php](https://github.com/ichinya/lekalo-php) через versioned provider API.

- Laravel-факты и сопоставления с семантической моделью: routes, controllers, Policies, FormRequest, storage mappings;
- интеграции Laramago и Laratesto (Mago + Laravel-пресеты, Testo + Laravel-окружение);
- библиотечный пакет с manifest и схемами — собственный демон не требуется.

Общий план отделения — [ichinya/lekalo#241](https://github.com/ichinya/lekalo/issues/241); зависимость на совместимую версию lekalo-php — [ichinya/lekalo#239](https://github.com/ichinya/lekalo/issues/239).
