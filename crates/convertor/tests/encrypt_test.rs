#[allow(unused)]
#[path = "./testkit.rs"]
mod testkit;

use crate::testkit::init_test;
use color_eyre::Result;
use convertor::common::encrypt::Encryptor;

#[test]
fn test_encrypt_and_decrypt() -> Result<()> {
    init_test();

    let secret = "abcdefg";
    let message = "This is a secret message.";

    let encryptor = Encryptor::new_with_label(secret, "test_encrypt_and_decrypt");
    // 加密
    let encrypted = encryptor.encrypt(message)?;
    insta::assert_snapshot!(encrypted, @"ANYFGS_Jgw8wigOWXNgSKeAOucYz2T9t9KwnZy7VTyJ3eYZIy7CQ31XR1KbqfuDmSxt7lTOgvq-70PrFNh42v-I");

    // 解密
    let decrypted = encryptor.decrypt(&encrypted)?;
    insta::assert_snapshot!(decrypted, @"This is a secret message.");

    Ok(())
}

#[test]
fn test_decrypt() -> Result<()> {
    init_test();

    let secret = "bppleman";
    let encryptor = Encryptor::new_with_label(secret, "test_decrypt");
    let token1 = encryptor.encrypt("http://127.0.0.1:64287/subscription?token=bppleman")?;
    let token2 = encryptor.encrypt("http://127.0.0.1:65019/subscription?token=bppleman")?;

    let decryptor = Encryptor::new_with_label(secret, "test_decrypt");

    let decrypted = decryptor.decrypt(&token1)?;
    println!("{}", decrypted);
    insta::assert_snapshot!(decrypted, @"http://127.0.0.1:64287/subscription?token=bppleman");

    let decrypted = decryptor.decrypt(&token2)?;
    insta::assert_snapshot!(decrypted, @"http://127.0.0.1:65019/subscription?token=bppleman");

    Ok(())
}

#[test]
fn deterministic_first_token_by_label() -> Result<()> {
    let secret = "my-secret";

    let e1 = Encryptor::new_with_label(secret, "case:hello:v1");
    let t1 = e1.encrypt("hello")?;

    let e2 = Encryptor::new_with_label(secret, "case:hello:v1");
    let t1_again = e2.encrypt("hello")?;

    assert_eq!(t1, t1_again);

    let p = e2.decrypt(&t1)?;
    assert_eq!(p, "hello");

    Ok(())
}

#[test]
fn deterministic_advances_per_instance() -> Result<()> {
    let secret = "my-secret";
    let e = Encryptor::new_with_label(secret, "case:seq:v1");

    let t1 = e.encrypt("hello")?;
    let t2 = e.encrypt("hello")?;
    assert_ne!(t1, t2);
    Ok(())
}

#[test]
fn long_secret_suffix_affects_key() -> Result<()> {
    let _ = init_test();

    let secret = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let enc = Encryptor::new_random(secret);
    let string = enc.encrypt("bppleman")?;
    assert_eq!(enc.decrypt(&string)?, "bppleman");

    let enc2 =
        Encryptor::new_random("viyExNQmEkwqCsc@sx!XF4A_DexX2HB-DYmPjYosr2R*Rd9gT_DW4LCXKRdKdATd4xokWscG8fAiPYUMLk7LUE6LwUkn.oNDjVhV");
    assert!(enc2.decrypt(&string).is_err());

    let enc3 = Encryptor::new_random("viyExNQmEkwqCsc@sx!XF4A_DexX2HB-DYmPjYosr2R*Rd9gT_DW4LCXKRdKdATd4xokWscG8fAiPYUMLk7LUE6LwUkn");
    let string = enc3.encrypt("bppleman")?;
    assert!(enc2.decrypt(&string).is_err());
    assert_eq!(enc3.decrypt(&string)?, "bppleman");

    Ok(())
}
