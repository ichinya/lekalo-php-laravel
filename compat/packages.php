<?php
declare(strict_types=1);

/** Locate packages beside the checkout, or inside a provisioned host. */
$rules = dirname(__DIR__);
if (!is_file($rules . '/lekalo.php-build.json')) {
    $rules = __DIR__ . '/../../tests/external/lekalo-php-laravel';
}
$rules = realpath($rules);
$runtimeSrc = getenv('LEKALO_PHP_SRC');
$runtime = $runtimeSrc === false || $runtimeSrc === ''
    ? ($rules === false ? false : realpath(dirname($rules) . '/lekalo-php'))
    : realpath(dirname($runtimeSrc));
if ($rules === false || $runtime === false || !is_file($runtime . '/build-lib.php')) {
    throw new RuntimeException('php-runtime-package-unavailable');
}
return [$runtime, $rules];
