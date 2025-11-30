use sha2::{Digest, Sha256};

include!("src/key.rs");

fn main() {
    if Sha256::digest(DECRYPTION_KEY)[..] != hex!("AF7C2AC4303C74CAE3E08125619DF375017C19C2FF5B61DD5682506E5F8200E4")
    {
        panic!("Decryption key hash does not match expected value!");
    }

    if Sha256::digest(DECRYPTION_IV)[..] != hex!("A102074E1A2025BFA2276B80FCE078A103A87665EA732FC28F921605F5120E15") 
    {
        panic!("Decryption IV hash does not match expected value!");
    }
}