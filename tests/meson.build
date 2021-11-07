test01 = executable('test01', 'test01.c')
test('test01', test01,
  env: {'LD_PRELOAD': fstracer.full_path()},
)

test02 = executable('test02', 'test02.c')
test('test02', test02,
  env: {'LD_PRELOAD': fstracer.full_path()},
)

test03 = executable('test03', 'test03.c')
test('test03', test03,
  env: {
    'LD_PRELOAD': fstracer.full_path(),
    'FSTRACER_OUTPUT': '/tmp/fstracer_test03',
  },
)
