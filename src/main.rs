extern crate rand_distr;

use std::io::Write;
use std::fs::{File, create_dir_all};
use std::io::BufWriter;

use rand_distr::{Normal, Distribution, Uniform};
use std::time::Instant;
use std::cmp::Ord;
use std::io::Read;
use std::collections::BTreeSet;

use uint::{u40,u48};
use uint::Typable;

fn main() {

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        println!("Bitte verwende {} <u40|48|u64> <normal|uniform> <max 2er-potenz>", args[0]);
        return;
    }

    if args[3].parse::<u64>().is_err() {
        println!("Bitte verwende {} <u40|48|u64> <normal|uniform> <max 2er-potenz>", args[0]);
        return;
    }

	match args[1].as_ref() {
		"u40" => stage1::<u40>(args),
		"u48" => stage1::<u48>(args),
		"u64" => stage1::<u64>(args),
		_ => println!("Bitte verwende {} <u40|48|u64> <normal|uniform> <max 2er-potenz>",args[0]),
    }
}

fn stage1<T: Typable + std::fmt::Display + Ord + Copy + Into<u64> + From<u64>>(args: Vec<String>) {
    let gen_start = Instant::now();
    match args[2].as_ref() {
        "normal" => {
            generate_normal_distribution::<T>(args[3].parse::<u64>().unwrap());
            println!("Normalverteilung erzeugt in {} Sekunden",gen_start.elapsed().as_secs());
        },
        "uniform" => {
            println!("Starte generierung der zufälligen Werte");
            generate_uniform_distribution::<T>(args[3].parse::<u64>().unwrap());
            println!("Gleichverteilung erzeugt in {} Sekunden",gen_start.elapsed().as_secs());
        }
        _ => {
            println!("Bitte verwende {} <u40|48|u64> <normal|uniform> <max 2er-potenz>",args[0]);
        }
    };
}

/// Diese Methode generiert 2^`exponent`viele unterschiedliche sortierte Zahlen vom Typ u40, u48 und u64.AsMut
/// Dabei werden Dateien von 2^0 bis hin zu 2^`exponent` angelegt.
fn generate_uniform_distribution<T: Typable + Ord + std::fmt::Display + Copy + Into<u64> + From<u64>>(exponent: u64) {
    // Erzeugt die testdata Directorys, falls diese noch nicht existieren.
    create_dir_all(format!("./testdata/uniform/{}/",T::TYPE)).unwrap();
 
    let max_value = (1u64<<exponent) as usize;

    let between = Uniform::from(0u64..(T::max_value()).into());
    let mut rng = rand::thread_rng();

    let mut memory: BTreeSet<T> = BTreeSet::new(); 
    let mut result = Vec::with_capacity(max_value);
    for _ in 0..max_value {
        let mut random_val = between.sample(&mut rng);
        while memory.contains(&T::from(random_val)) {
            random_val = between.sample(&mut rng);
        }

        let val: T = random_val.into();
        memory.insert(val);
        result.push(val);
    }

    // 2^0 wird ausgelassen, da die Verarbeitung von genau einem Element im späteren Programmablauf problematisch wäre.
    for i in 1..exponent {
        let cut = result.len() - (max_value - (1u64<<i) as usize); 
        let result = &mut result[..cut];
        result.sort();

        write_to_file(format!("./testdata/uniform/{}/2^{}.data",T::TYPE, i),result).unwrap();
    }

    result.sort();
    result.dedup();
    assert!(result.len() == max_value);
    create_input::<T>("uniform",&result[..]);
    write_to_file(format!("./testdata/uniform/{}/2^{}.data",T::TYPE, exponent),&result[..]).unwrap();
}

/// Diese Methode generiert 2^`exponent`viele normalverteilte sortierte Zahlen vom Typ u40, u48 und u64.AsMut
/// Dabei werden Dateien von 2^0 bis hin zu 2^`exponent` angelegt.
fn generate_normal_distribution<T: Typable + num::Bounded + Ord + std::fmt::Display + Copy + Into<u64> + From<u64>>(exponent: u64) {
    let mean = (1u64<<std::mem::size_of::<T>()*8-1) as f64;
    // Laut https://en.wikipedia.org/wiki/Standard_deviation#/media/File:Standard_deviation_diagram.svg deckt die Normalverteilung 
    // ein Achtel des gültigen Wertebereich ab.
    let deviation: f64 = mean/32.;
    // Dieses Bitarray hat für jeden möglichen u40 einen Bit als Eintrag, der angibt, ob dieser Wert bereits gesammelt wurde
    //let mut memory = vec![0u64;(T::max_value().into()/64) as usize];
    // Erzeugt die testdata Directorys, falls diese noch nicht existieren.
    create_dir_all(format!("./testdata/normal/{}/",T::TYPE)).unwrap();

    let normal = Normal::new(mean, deviation).unwrap();
    let max_value = (1u64<<exponent) as usize;
    let mut rng = rand::thread_rng();
    let mut memory: BTreeSet<T> = BTreeSet::new(); 
    let mut result = Vec::with_capacity(max_value);
    for _ in 0..max_value {
        let mut random_val = normal.sample(&mut rng);
        while random_val < 0.0 || (random_val as u64) > T::max_value().into() || memory.contains(&T::from(random_val as u64)) {
            random_val = normal.sample(&mut rng);
        }

        let val: T = (random_val as u64).into();
        memory.insert(val);
        result.push(val);
    }



    // 2^0 wird ausgelassen, da die Verarbeitung von genau einem Element im späteren Programmablauf problematisch wäre.
    for i in 1..exponent {
        let cut = result.len() - (max_value - (1u64<<i) as usize); 
        let result = &mut result[..cut];
        result.sort();

        write_to_file(format!("./testdata/normal/{}/2^{}.data", T::TYPE, i),result).unwrap();
    }

    result.sort();
    result.dedup();
    assert!(result.len() == max_value);

    create_input::<T>("normal",&result[..]);
    write_to_file(format!("./testdata/normal/{}/2^{}.data", T::TYPE, exponent),&result[..]).unwrap();
}

/// Serializiert den übergebenen Vector und schreibt diesen in eine Datei namens `name`.
fn write_to_file<T: Typable + Copy + Into<u64>>(name: String, val: &[T]) -> std::io::Result<()>{
    let mut buf = BufWriter::new(File::create(name).unwrap());
    buf.write_all(&val.len().to_le_bytes())?;
    for &v in val {
        let v: u64 = v.into();
        buf.write_all(&v.to_le_bytes()[..std::mem::size_of::<T>()])?;
    }
    buf.flush()?;
    Ok(())
}

fn create_input<E: Typable + Into<u64> + Copy + std::fmt::Display + From<u64> + Into<u64>>(data: &str, values: &[E]) {
    std::fs::create_dir_all(format!("input/pred/{}/{}/", data, E::TYPE)).unwrap();

    let values_len = values.len();

    let test_values = get_test_values(E::from(values[0].into()+1u64),values[values_len-1]);

    write_to_file(format!("input/pred/{}/{}/min{}_max{}.data",data, E::TYPE, values[0],values[values_len-1]).to_string(), &test_values).unwrap();

}

fn get_test_values<E: Typable + Copy + From<u64> + Into<u64> >(min: E, max: E) -> Vec<E> {
    let between = Uniform::from(0u64..max.into());
    let mut rng = rand::thread_rng();
    let mut result = Vec::with_capacity(100000);
    for _ in 0..100000 {
        let random_val = between.sample(&mut rng);

        let val: E = random_val.into();
        result.push(val);
    }
    result
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