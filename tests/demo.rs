extern crate rand;
extern crate ed25519_dalek;
extern crate ecies_ed25519;
use rand::rngs::OsRng;
use shamir::SecretData;

#[test]
fn test_sign_dalek() {
    use ed25519_dalek::{
        Signer,
        Verifier,
    };

    let mut csprng = OsRng{};
    let keypair: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut csprng);

    let message: &[u8] = b"Hello world!";
    let signature: ed25519_dalek::Signature = keypair.sign(message);

    assert!(keypair.verify(message, &signature).is_ok());

    let public_key: ed25519_dalek::PublicKey = keypair.public;
    assert!(public_key.verify(message, &signature).is_ok());
}

#[test]
fn test_sign_openssl() {
    use openssl::sign::{Signer, Verifier};
    use openssl::ec::{EcKey,EcGroup, EcPoint};
    use openssl::nid::Nid;
    use openssl::pkey::PKey;
    use openssl::hash::MessageDigest;

    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
    let key = EcKey::generate(&group).unwrap();
    let mut ctx = openssl::bn::BigNumContext::new().unwrap();
    
    println!("private eckey = {:?}", key.private_key());

    let bytes = key.public_key().to_bytes(&group,
        openssl::ec::PointConversionForm::COMPRESSED, &mut ctx).unwrap();
    
    println!("public key = {:?}", bytes);

    let public_key = EcPoint::from_bytes(&group, &bytes, &mut ctx).unwrap();
    let ec_key = EcKey::from_public_key(&group, &public_key).unwrap();

    assert!(ec_key.check_key().is_ok());

    let message: &[u8] = b"Hello world!";
    // Sign the data
    let keypair = PKey::from_ec_key(key).unwrap();
    let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
    signer.update(message).unwrap();
    let signature = signer.sign_to_vec().unwrap();

    // Verify the data
    let mut verifier = Verifier::new(MessageDigest::sha256(), &keypair).unwrap();
    assert!(verifier.verify_oneshot(&signature, message).unwrap());
}

#[test]
fn test_parse_openssl() {
    use openssl::ec::{EcKey,EcGroup, EcPoint};
    use openssl::nid::Nid;
    use openssl::symm::Cipher;

    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
    let key = EcKey::generate(&group).unwrap();
    
    let password = b"secret_pwd";
    let pem = key.private_key_to_pem_passphrase(Cipher::aes_128_cbc(), password).unwrap();
    let pem_str = String::from_utf8(pem).unwrap();
    println!("PEM = {:?}", pem_str);
    
    let wrong_pwd = b"wrong_pwd";
    assert!(EcKey::private_key_from_pem_passphrase(pem_str.as_bytes(), wrong_pwd).is_err());
    assert!(EcKey::private_key_from_pem_passphrase(pem_str.as_bytes(), password).is_ok());

    let mut ctx = openssl::bn::BigNumContext::new().unwrap();
    let bytes = key.public_key().to_bytes(&group, openssl::ec::PointConversionForm::COMPRESSED, &mut ctx).unwrap();
    let point = EcPoint::from_bytes(&group, &bytes, &mut ctx).unwrap();
    EcKey::from_public_key(&group, &point).unwrap();
}

#[test]
fn test_encrypt_openssl() {
    use openssl::encrypt::{Encrypter, Decrypter};
    use openssl::rsa::{Rsa, Padding};
    use openssl::pkey::PKey;
    
    // Generate a keypair
    let keypair = Rsa::generate(2048).unwrap();
    let keypair = PKey::from_rsa(keypair).unwrap();
    
    let data = b"hello, world!";
    
    // Encrypt the data with RSA PKCS1
    let mut encrypter = Encrypter::new(&keypair).unwrap();
    encrypter.set_rsa_padding(Padding::PKCS1).unwrap();
    // Create an output buffer
    let buffer_len = encrypter.encrypt_len(data).unwrap();
    let mut encrypted = vec![0; buffer_len];
    // Encrypt and truncate the buffer
    let encrypted_len = encrypter.encrypt(data, &mut encrypted).unwrap();
    encrypted.truncate(encrypted_len);
    
    // Decrypt the data
    let mut decrypter = Decrypter::new(&keypair).unwrap();
    decrypter.set_rsa_padding(Padding::PKCS1).unwrap();
    // Create an output buffer
    let buffer_len = decrypter.decrypt_len(&encrypted).unwrap();
    let mut decrypted = vec![0; buffer_len];
    // Encrypt and truncate the buffer
    let decrypted_len = decrypter.decrypt(&encrypted, &mut decrypted).unwrap();
    decrypted.truncate(decrypted_len);
    assert_eq!(&*decrypted, data);
}

#[test]
fn test_encrypt_ecies() {
    let mut csprng = OsRng{};
    let (secret, public) = ecies_ed25519::generate_keypair(&mut csprng);
    let message: &[u8] = b"Hello world!";
    // Encrypt the message with the public key such that only the holder of the secret key can decrypt.
    let encrypted = ecies_ed25519::encrypt(&public, message, &mut csprng).unwrap();
    // Decrypt the message with the secret key
    let decrypted = ecies_ed25519::decrypt(&secret, &encrypted).unwrap();
    assert_eq!(message, decrypted);
}

#[test]
fn test_password() {
    use pwbox::{Eraser, ErasedPwBox, Suite, sodium::Sodium};
    use rand_core::OsRng;

    let pwd = b"password";
    let data =  b"Hello world!";

    // Create a new box.
    let mut csprng = OsRng{};
    let pwbox = Sodium::build_box(&mut csprng)
                .seal(pwd, data).unwrap();

    // Serialize box.
    let mut eraser = Eraser::new();
    eraser.add_suite::<Sodium>();
    let erased: ErasedPwBox = eraser.erase(&pwbox).unwrap();
    let code = serde_json::to_string_pretty(&erased).unwrap();

    // Deserialize box back.
    let restored: ErasedPwBox = serde_json::from_str(&code).unwrap();
    let plaintext = eraser.restore(&restored).unwrap().open(pwd).unwrap();
    assert_eq!(&*plaintext, data);
}

#[test]
fn test_shamir() {
    let msg = "Hello world!";
    let needed = 3;

    let secret_data = SecretData::with_secret(msg, needed);
    println!("{:?}", secret_data.secret_data);

    let share1 = secret_data.get_share(1);
    let share2 = secret_data.get_share(2);
    let share3 = secret_data.get_share(3);

    let mut recovered = SecretData::recover_secret(3, vec![share1, share2, share3]).unwrap();
    assert_eq!(recovered, msg);

    let share4 = secret_data.get_share(4);
    let share5 = secret_data.get_share(5);
    let share6 = secret_data.get_share(6);

    recovered = SecretData::recover_secret(3, vec![share4, share5, share6]).unwrap();
    assert_eq!(recovered, msg);
}