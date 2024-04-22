use rust_embed::RustEmbed;

#[derive(RustEmbed, Debug)]
#[folder = "../../client/build"]
#[include = "*.html"]
#[include = "*.json"]
#[include = "*.txt"]
#[include = "*.ico"]
pub struct Asset;

#[derive(RustEmbed, Debug)]
#[folder = "../../client/build/static/"]
#[include = "js/*"]
#[include = "css/*"]
#[include = "media/*"]
pub struct Statics;

#[derive(RustEmbed, Debug)]
#[folder = "../../contracts/circuits/coin_withdraw_js"]
#[include = "*.wasm"]
#[include = "*.js"]

pub struct CircuitsStatics;

#[derive(RustEmbed, Debug)]
#[folder = "../../contracts/circuits"]
#[include = "*.zkey"]

pub struct ZkStatics;

#[derive(RustEmbed, Debug)]
#[folder = "../../."]
#[include = "*.json"]
#[include = "*.dat"]

pub struct ConfigAsset;
