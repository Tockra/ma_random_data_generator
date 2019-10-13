extern crate rand_distr;

use std::io::Write;
use std::fs::{File, create_dir_all};
use std::io::BufWriter;
use rand_pcg::Mcg128Xsl64;
use rand::seq::IteratorRandom;
use rand_distr::{Normal, Distribution};
use std::time::Instant;
use std::cmp::Ord;
use std::io::Read;

use uint::u40;
use uint::Typable;


const SEED: u128 = 0xcafef00dd15ea5e5;

fn main() {

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Bitte genau ein Argument übergeben!");
    }

    let gen_start = Instant::now();

    match args[1].as_ref() {
        "normal_komplett" => {
            generate_normal_distribution::<u40>(32, (1u64<<39) as f64, (1u64<<37) as f64, "bereich_komplett");
            println!("Normalverteilung erzeugt in {} Sekunden",gen_start.elapsed().as_secs());
        },
        "normal_viertel" => {
            generate_normal_distribution::<u40>(32, (1u64<<39) as f64, (1u64<<35) as f64, "bereich_viertel");
            println!("Normalverteilung erzeugt in {} Sekunden",gen_start.elapsed().as_secs());
        },
        "uniform" => {
            println!("Starte generierung der zufälligen Werte");
            generate_uniform_distribution::<u40>(32);
            println!("Gleichverteilung erzeugt in {} Sekunden",gen_start.elapsed().as_secs());
        }
        _ => {
            println!("Bitte verwende {} <normal_komplett|normal_viertel|uniform>",args[0]);
        }
    };
}

/// Diese Methode generiert 2^`exponent`viele unterschiedliche sortierte Zahlen vom Typ u40, u48 und u64.AsMut
/// Dabei werden Dateien von 2^0 bis hin zu 2^`exponent` angelegt.
fn generate_uniform_distribution<T: Typable + Ord + Copy + Into<u64> + From<u64>>(exponent: u64) {
    // Erzeugt die testdata Directorys, falls diese noch nicht existieren.
    create_dir_all(format!("./testdata/uniform/{}/",T::TYPE)).unwrap();

    let mut state = Mcg128Xsl64::new(SEED);
    let max_value = (1u64<<exponent) as usize;
    let mut result: Vec<T> = (0u64..(T::max_value()).into()).map(|v| T::from(v)).choose_multiple(&mut state, max_value);

    // 2^0 wird ausgelassen, da die Verarbeitung von genau einem Element im späteren Programmablauf problematisch wäre.
    for i in 1..exponent {
        let cut = result.len() - (max_value - (1u64<<i) as usize); 
        let result = &mut result[..cut];
        result.sort();

        write_to_file(format!("./testdata/uniform/{}/2^{}.data",T::TYPE, i),result).unwrap();
    }

    result.sort();
    write_to_file(format!("./testdata/uniform/{}/2^{}.data",T::TYPE, exponent),&result[..]).unwrap();
}

/// Diese Methode generiert 2^`exponent`viele normalverteilte sortierte Zahlen vom Typ u40, u48 und u64.AsMut
/// Dabei werden Dateien von 2^0 bis hin zu 2^`exponent` angelegt.
fn generate_normal_distribution<T: Typable + Ord + Copy + Into<u64> + From<u64>>(exponent: u64, mean: f64, deviation: f64, name: &str) {

    // Dieses Bitarray hat für jeden möglichen u40 einen Bit als Eintrag, der angibt, ob dieser Wert bereits gesammelt wurde
    let mut memory = vec![0u64;(T::max_value().into()/64) as usize];
    // Erzeugt die testdata Directorys, falls diese noch nicht existieren.
    create_dir_all(format!("./testdata/normal/{}/{}/",name,T::TYPE)).unwrap();

    let normal = Normal::new(mean, deviation).unwrap();
    let max_value = (1u64<<exponent) as usize;
    let mut rng = rand::thread_rng();
    let mut result: Vec<T> = Vec::with_capacity(max_value); 
    for _ in 0..max_value {
        let mut random_val = normal.sample(&mut rng);
        while random_val < 0.0 || (random_val as u64) > T::max_value().into() || contains(random_val as u64, &memory) {
            random_val = normal.sample(&mut rng);
        }

        result.push((random_val as u64).into());
        set_value(random_val as u64, &mut memory);
        
    }

    // 2^0 wird ausgelassen, da die Verarbeitung von genau einem Element im späteren Programmablauf problematisch wäre.
    for i in 1..exponent {
        let cut = result.len() - (max_value - (1u64<<i) as usize); 
        let result = &mut result[..cut];
        result.sort();

        write_to_file(format!("./testdata/normal/{}/{}/2^{}.data",name, T::TYPE, i),result).unwrap();
    }

    result.sort();
    write_to_file(format!("./testdata/normal/{}/{}/2^{}.data",name, T::TYPE, exponent),&result[..]).unwrap();
}

/// Serializiert den übergebenen Vector und schreibt diesen in eine Datei namens `name`.
fn write_to_file<T: Typable + Copy + Into<u64>>(name: String, val: &[T]) -> std::io::Result<()>{
    let mut buf = BufWriter::new(File::create(name).unwrap());
    buf.write_all(&val.len().to_le_bytes())?;
    for &v in val {
        let v: u64 = v.into();
        buf.write_all(&v.to_le_bytes()[..std::mem::size_of::<T>()])?;
    }
    Ok(())
}

pub fn read_from_file<T: Typable + From<u64> + Copy>(name: String) -> std::io::Result<Vec<T>> {
    let mut input = File::open(name.clone())?;
    let mut lenv = Vec::new();
    std::io::Read::by_ref(&mut input).take(std::mem::size_of::<usize>() as u64).read_to_end(&mut lenv)?;
    let mut len: [u8; std::mem::size_of::<usize>()] = [0; std::mem::size_of::<usize>()];
    for (i,b) in lenv.iter().enumerate() {
        len[i] = *b;
    }
    let len: usize = usize::from_le_bytes(len);

    assert!(len == (std::fs::metadata(name)?.len() as usize - std::mem::size_of::<usize>())/ std::mem::size_of::<T>());

    let mut values: Vec<T> = Vec::with_capacity(len);
    while values.len() != len {
        let mut buffer = Vec::with_capacity(std::mem::size_of::<T>());
        std::io::Read::by_ref(&mut input).take(std::mem::size_of::<T>() as u64).read_to_end(&mut buffer)?;
        let mut next_value: u64 = 0;
        for i in 0..buffer.len() {
            next_value |= (buffer[i] as u64) << (8*i);
        }

        values.push(T::from(next_value));
    }
    Ok(values)
}

#[inline]
fn contains(val: u64, memory: &Vec<u64>) -> bool {
    let index = (val/64) as usize;
    let in_index = val%64;
    let mask = 1u64<<(63-in_index);
    (memory[index] & mask) != 0
}

#[inline]
fn set_value(val: u64, memory: &mut Vec<u64>) {
    let index = (val/64) as usize;
    let in_index = val%64;
    let mask = 1u64<<(63-in_index);
    memory[index] = memory[index] | mask;
}