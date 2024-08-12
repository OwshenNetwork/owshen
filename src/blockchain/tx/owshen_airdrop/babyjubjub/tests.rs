use super::*;

#[test]
fn test_twisted_edwards_curve_ops() {
    // ((2G) + G) + G
    let mut a = BASE.double().unwrap();
    a.add_assign(&BASE).unwrap();
    a.add_assign(&BASE).unwrap();

    // 2(2G)
    let b = BASE.double().unwrap().double().unwrap();

    assert_eq!(a, b);

    // G + G + G + G
    let mut c = *BASE;
    c.add_assign(&BASE).unwrap();
    c.add_assign(&BASE).unwrap();
    c.add_assign(&BASE).unwrap();

    assert_eq!(b, c);

    // Check if projective points are working
    let mut pnt1 = BASE.to_projective().double().double();
    pnt1.add_assign(&BASE.to_projective()).unwrap();
    let mut pnt2 = BASE.double().unwrap().double().unwrap();
    pnt2.add_assign(&BASE).unwrap();

    assert_eq!(pnt1.to_affine().unwrap(), pnt2);
}

#[test]
fn test_jubjub_public_key_compression() {
    let p1 = BASE.multiply(&Fp::from(123_u64)).unwrap();
    let p2 = p1.compress().decompress().unwrap();

    assert_eq!(p1, p2);
}

#[test]
fn test_jubjub_signature_verification() {
    let randomness = Fp::from(2345);
    let sk = PrivateKey(Fp::from(12345));
    let pk = sk.to_pub().unwrap();
    let msg = Fp::from(123456);
    let fake_msg = Fp::from(123457);
    let sig = sk.sign(randomness, msg).unwrap();

    assert!(pk.verify(msg, &sig).unwrap());
    assert!(!pk.verify(fake_msg, &sig).unwrap());
}
