# The Owshen 🌊

Owshen is a privacy platform built for EVM-based blockchains. Owshen gathers multiple ideas around cryptocurrency privacy solutions in a single place to provide ultimate privacy.

Using Owshen you can get a ***fixed*** Owshen address and start transacting with users inside/outside of the platform, without exposing:

1. **Source** (Spend your coins using Zcash/TornadoCash-style merkle inclusion proofs, along with nullifiers)
2. **Destination** (Monero-style stealth-addresses are generated each time you send your coins to someone)
3. **Token/Amount** (These values are obfuscated and only the sender and receiver, who know a shared-secret, will be able to decode them)

Join our Discord: https://discord.gg/owshen

## Usage

 - Clone the project `git clone https://github.com/OwshenNetwork/owshen --recurse-submodules`
 - If you already cloned the project without the cloning submodules first running: `git submodule update --init --recursive`
 - The option `--remote` was added to support updating to latest tips of remote branches: `git submodule update --recursive --remote`
 - Install Rust language: `curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh`
 - Install Foundry: `https://book.getfoundry.sh/getting-started/installation`
 - Install dependencies: `apt-get install nodejs npm libgmp3-dev nasm nlohmann-json3-dev`
 - Install Circom/SnarkJS: `npm i -g snarkjs circom`
 - Install Owshen: `cd owshen && make install`
 - For installing client dependencies we need to go to client route and: `yarn` or `npm install`  
 - Running proper Ganache localhost network: `ganache-cli -d --db chain`
 (We need to import first account from Ganache to metamask for local testing)
 - Initialize your pub/priv keys and deploying dependencies by running  `cargo run -- init --endpoint http://127.0.0.1:8545 --db test.json` (Your keys will be saved in `~/.owshen-wallet.json` - also you can running this command multiple times for testing purpose)
 - Run the wallet (GUI): `cargo run -- wallet --port 9000 --db test.json`

## How? 🤔

Owshen Platform is basically a smart-contract maintaining a Sparse-Merkle-Tree, very similar to TornadoCash, with one big difference. Instead of commitments (Which are hashes of secret values), elliptic-curve points (Public-keys) are stored in the leaves, and one can only spend a coin in case he proves that he knows a private-key $s$, where $s \times G$ ($G$ is a commonly agreed generator point) is a point that exists in the tree (Through a merkle-proof fed in a Zero-Knowledge proof circuit).

Fixed addresses are bad for the destination's privacy, a TornadoCash-style pool will only allow you to hide the sender, but everyone watching from outside can see that money is being sent to the receiver. We may solve this problem by requiring the receiver to generate a new address whenever he wants to receive a coin, but this would require the receiver to be online all the time. In case the receiver is someone accepting donations, it's easiest for him to announce a fixed address for receiving the donations.

Stealth-addresses solve this problem already: instead of requiring the receiver to generate a new address everytime he wants to receive the coin, we will let the sender derive stealth public-keys, from the receiver's master public key!

The sender will generate a random scalar $r$, and will broadcast the point $r \times G$ publicly. In this case, $s \times r \times G$ is a shared-secret between the sender and the receiver (Very similar to Diffie-Hellman key-exchange algorithm).  $s \times r \times G$ is an elliptic curve point, we can convert it to a scalar using a hash function, so that it can be used a private-key. The sender will send the coin to $(hash(s \times r \times G) + s)\times G$ instead of $s \times G$, and then the receiver would be able to detect incoming transactions and derive the corresponding private-keys of stealth-addresses: $hash(s \times r \times G) + s$.

### Owshen's merkle-tree structure :evergreen_tree: 

As previosuly said, a Sparse-Merkle-Tree is being maintained in Owshen platform's smart-contract, where each leaf is:

$hash({pub}_x,{pub}_y,token,amount)$

One can spend/withdraw a coin in the merkle-tree by proving:

 > I know a private key $s$ (Private), where there exists a leaf in tree with public-key $s \times G$, holding $amount$ of $token$.

After each send, an event will be emitted, providing the data needed for the receiver to recognize his incoming transactions:

```solidity=
event Sent(
    Point pub_key, // g^(hash(g^sr) + s)
    Point ephemeral, // g^r
    uint256 encoded_token, // token + hash(g^sr)
    uint256 encoded_amount // amount + hash(g^sr)
);
```

The shared secret between the sender and receiver is $hash(g^{sr})$. We can add the shared-secret to the token-id and amount in order to obfuscate them. ($p$ is the field-size)

${token}_{encoded} = ({token} + hash(g^{sr})) \mod p$

${amount}_{encoded} = ({amount} + hash(g^{sr})) \mod p$

The receiver may subtract the shared secret from token/amount to calculate the leaf's actual token/amount and try to calculate the commitment. If the commitment he has calculated is equal with the commitment submitted on-chain, then the coin is for him and he can derive the private-key needed for spending that coin.
