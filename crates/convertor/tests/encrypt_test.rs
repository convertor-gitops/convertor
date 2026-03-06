#[allow(unused)]
mod testkit;

use crate::testkit::init_test;
use convertor::common::encrypt::Encryptor;

#[test]
fn test_encrypt_and_decrypt() -> color_eyre::Result<()> {
    init_test();

    let secret = b"abcdefg"; // 密钥必须是32字节
    let message = "This is a secret message.";

    let encryptor = Encryptor::new_with_label(secret, "test_encrypt_and_decrypt");
    // 加密
    let encrypted = encryptor.encrypt(message)?;
    insta::assert_snapshot!(encrypted, @"ANYFGS_Jgw8wigOWXNgSKeAOucYz2T9t5jnnekqypYk7ii9lfZOsBccxAM_Ag8AKx7INgIkIcNhvrX5e6XL1YkI");

    // 解密
    let decrypted = encryptor.decrypt(&encrypted)?;
    insta::assert_snapshot!(decrypted, @"This is a secret message.");

    Ok(())
}

#[test]
fn test_decrypt() -> color_eyre::Result<()> {
    init_test();

    let secret = "bppleman";
    let encryptor = Encryptor::new_with_label(secret.as_bytes(), "test_decrypt");

    let decrypted = encryptor.decrypt(
        "qDbvzIt3DcfaQVl8UVdIjXck4D-42Eo3UN2hjcQ3B_IH9FI51WQX94QusyP4URwR4naCdMYFGV6aljrLzyNRhsJg9Cj55JszewkvSRXW5zMgUJCkai79FKZ4",
    )?;
    println!("{}", decrypted);
    insta::assert_snapshot!(decrypted, @"http://127.0.0.1:64287/subscription?token=bppleman");

    let decrypted = encryptor.decrypt(
        "qDbvzIt3DcfaQVl8UVdIjXck4D-42Eo3UN2hjcQ3B_IH9FI51WQX94QusiHxXxwR4naCdMYFGV6aljrLzyNRhsJg9Cj55Jszewk65g-J2hWsrxSAc1sHyTK1",
    )?;
    insta::assert_snapshot!(decrypted, @"http://127.0.0.1:65019/subscription?token=bppleman");

    Ok(())
}
