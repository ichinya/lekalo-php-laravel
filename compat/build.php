#!/usr/bin/env php
<?php
/** Transitional compatibility entry; package manifests own every source/resource. */
declare(strict_types=1);
[$runtime, $rules] = require __DIR__ . '/packages.php';
require $runtime . '/build-lib.php';
try {
    $args = array_slice($_SERVER['argv'] ?? [], 1);
    if ($args !== [] && $args !== ['--check']) { throw new RuntimeException('usage: build.php [--check]'); }
    $result = lekalo_build_compose(
        $runtime,
        [$rules],
        'lekalo-target-php-laravel',
        'php-laravel',
    );
    fwrite(STDOUT, json_encode(lekalo_build_publish($result, __DIR__ . '/adapter.php', $args === ['--check']), JSON_UNESCAPED_SLASHES | JSON_THROW_ON_ERROR) . "\n");
} catch (Throwable $error) {
    fwrite(STDERR, $error instanceof RuntimeException ? $error->getMessage() . "\n" : "composition-invalid\n");
    exit(1);
}
