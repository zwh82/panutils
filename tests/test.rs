
use assert_cmd::Command;

// tests/GCF_002012065.1_ASM201206v1_genomic.fna
// tests/GCF_006400955.1_ASM640095v1_genomic.fna

// fastixe
#[test]
fn test_cli1() {
    let mut cmd = Command::cargo_bin("panutils").unwrap();
    cmd.args(&[
        "fastixe", 
        "-i", "tests/GCF_002012065.1_ASM201206v1_genomic.fna",
        "--up",
    ])
    .assert()
    .success();
}

#[test]
fn test_cli2() {
    let mut cmd = Command::cargo_bin("panutils").unwrap();
    cmd.args(&[
        "fastixe", 
        "-i", "tests/GCF_002012065.1_ASM201206v1_genomic.fna",
        "--up",
        "-p", "GCF_002012065.1#0#"
    ])
    .assert()
    .success();
}

#[test]
fn test_cli3() {
    let mut cmd = Command::cargo_bin("panutils").unwrap();
    cmd.args(&[
        "fastixe", 
        "-s", "tests/GCF_002012065.1_ASM201206v1_genomic.fna", "tests/GCF_006400955.1_ASM640095v1_genomic.fna",
        "--up",
    ])
    .assert()
    .success();
}


#[test]
fn test_cli4() {
    let mut cmd = Command::cargo_bin("panutils").unwrap();
    cmd.args(&[
        "fastixe", 
        "-l", "tests/test_genome_list.txt",
        "--up",
    ])
    .assert()
    .success();
}

#[test]
fn test_cli5() {
    let mut cmd = Command::cargo_bin("panutils").unwrap();
    cmd.args(&[
        "fastixe", 
        "-d", "tests/",
        "--up",
    ])
    .assert()
    .success();
}

// gzip output
#[test]
fn test_cli6() {
    let mut cmd = Command::cargo_bin("panutils").unwrap();
    cmd.args(&[
        "fastixe", 
        "-d", "tests/",
        "-g",
        "--up",
    ])
    .assert()
    .success();
}

// merge and bgzip output
#[test]
#[cfg(feature = "c_ffi")]
fn test_cli7() {
    let mut cmd = Command::cargo_bin("panutils").unwrap();
    cmd.args(&[
        "fastixe", 
        "-d", "tests/",
        "-m",
        "-b",
        "--up",
    ])
    .assert()
    .success();
}

// merge and bgzip output and faidx
#[test]
#[cfg(feature = "c_ffi")]
fn test_cli8() {
    let mut cmd = Command::cargo_bin("panutils").unwrap();
    cmd.args(&[
        "fastixe", 
        "-d", "tests/",
        "-m",
        "-b",
        "-f",
        "--up",
    ])
    .assert()
    .success();
}

#[test]
fn test_cli9() {
    let mut cmd = Command::cargo_bin("panutils").unwrap();
    cmd.args(&[
        "fastixe", 
        "-d", "tests/",
        "-m",
        "-e", "test_merged.fa",
        "--up",
    ])
    .assert()
    .success();
}
