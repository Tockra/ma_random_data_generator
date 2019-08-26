# random_data_generator
This projects generates 2^i unique random u40 and u48 values. You can pass the `i` in the main function. You can find the u40 and u48 type here: https://github.com/Tockra/uint.

You need to generate the `../testdata/{u40,u48}/` folder (which one you need depends on the main method).
There will the generator generate files with 2^0, 2^1, ..., 2^i random, unique values. You can deserialize it with serde (https://crates.io/crates/serde) and get a Vec<u40> or Vec<u48> (depending on the source). 
