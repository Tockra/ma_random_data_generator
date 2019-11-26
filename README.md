# random_data_generator
Diese Bibliothek erzeugt die u40, u48 und u64 Testdaten in ./testdata/ . Dieser Ordner kann hinterher nach ma_eval_speed und ma_eval_space kopiert werden. Die u40 und u48 Implementierung kommt von hier: https://github.com/Tockra/uint .

Im Ordner ./testdata werden die Testdaten erzeugt,
Im Ordner ./input/ werden die Inputdaten für Vorgängeranfragen erzeugt (nach ma_eval_speed kopieren und den vorhandenen Ordner dort ersetzen)

Die Generierung hat folgende Syntax:
```bash
cargo run --release <type={u40,u48,u64}> <verteilung={normal,uniform}> <max 2er-potenz>
```

**<type={u40,u48,u64}>**
- u40 : 40-Bit-Integer
- u48 : 48-Bit-Integer
- u64 : 64-Bit-Integer

**<verteilung={normal,uniform}>**
- normal: Normalverteilung
- uniform: Gleichverteilung

**< max 2er-potenz >**
- Größter 2er-Potenz der Testdaten, die erzeugt werden sollen