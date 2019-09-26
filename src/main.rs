extern crate serde;
extern crate rmp_serde as rmps;
extern crate rand_distr;

use serde::{Serialize};
use rmps::{Serializer};
use std::fs::{File, create_dir_all};
use std::io::BufWriter;
use rand_pcg::Mcg128Xsl64;
use rand::seq::IteratorRandom;
use rand_distr::{Normal, Distribution};
use std::time::Instant;
use std::cmp::Ord;

use uint::u40;
use uint::Typable;


const SEED: u128 = 0xcafef00dd15ea5e5;

fn main() {
    let gen_start = Instant::now();
    println!("Starte generierung der zufälligen Werte");
    generate_uniform_distribution::<u40>(32);
    println!("Gleichverteilung erzeugt in {} Sekunden",gen_start.elapsed().as_secs());
    let gen_normal = Instant::now();
    generate_normal_distribution::<u40>(32, (1u64<<39) as f64, (1u64<<37) as f64, "bereich_komplett");
    println!("Normalverteilung erzeugt in {} Sekunden",gen_normal.elapsed().as_secs());

    let gen_normal = Instant::now();
    generate_normal_distribution::<u40>(32, (1u64<<39) as f64, (1u64<<35) as f64, "bereich_viertel");
    println!("Normalverteilung erzeugt in {} Sekunden",gen_normal.elapsed().as_secs());

    

    println!("Gesamtlaufzeit: {} Sekunden",gen_start.elapsed().as_secs());
}

/// Diese Methode generiert 2^`exponent`viele unterschiedliche sortierte Zahlen vom Typ u40, u48 und u64.AsMut
/// Dabei werden Dateien von 2^0 bis hin zu 2^`exponent` angelegt.
fn generate_uniform_distribution<T: Typable + Serialize + Ord + Copy + Into<u64> + From<u64>>(exponent: u64) {
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

        write_to_file(format!("./testdata/uniform/{}/2^{}.data",T::TYPE, i),&result.to_vec());
    }

    result.sort();
    write_to_file(format!("./testdata/uniform/{}/2^{}.data",T::TYPE, exponent),&result);
}

/// Diese Methode generiert 2^`exponent`viele normalverteilte sortierte Zahlen vom Typ u40, u48 und u64.AsMut
/// Dabei werden Dateien von 2^0 bis hin zu 2^`exponent` angelegt.
fn generate_normal_distribution<T: Typable + Serialize + Ord + Copy + Into<u64> + From<u64>>(exponent: u64, mean: f64, deviation: f64, name: &str) {
    // Erzeugt die testdata Directorys, falls diese noch nicht existieren.
    create_dir_all(format!("./testdata/normal/{}/{}/",name,T::TYPE)).unwrap();

    let normal = Normal::new(mean, deviation).unwrap();
    let max_value = (1u64<<exponent) as usize;
    let mut rng = rand::thread_rng();
    let mut result: Vec<T> = Vec::with_capacity(max_value); 
    for _ in 0..max_value {
        let mut random_val = normal.sample(&mut rng);
        while random_val < 0.0 || (random_val as u64) > T::max_value().into() || result.contains(&T::from(random_val as u64)) {
            random_val = normal.sample(&mut rng);
        }

        result.push((random_val as u64).into());
        
    }

    // 2^0 wird ausgelassen, da die Verarbeitung von genau einem Element im späteren Programmablauf problematisch wäre.
    for i in 1..exponent {
        let cut = result.len() - (max_value - (1u64<<i) as usize); 
        let result = &mut result[..cut];
        result.sort();

        write_to_file(format!("./testdata/normal/{}/{}/2^{}.data",name, T::TYPE, i),&result.to_vec());
    }

    result.sort();
    write_to_file(format!("./testdata/normal/{}/{}/2^{}.data",name, T::TYPE, exponent),&result);
}

/// Serializiert den übergebenen Vector und schreibt diesen in eine Datei namens `name`.
fn write_to_file<T: Typable + Serialize>(name: String, val: &Vec<T>) {
    let mut buf = BufWriter::new(File::create(name).unwrap());
    val.serialize(&mut Serializer::new(&mut buf)).unwrap();
}