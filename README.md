# The Owshen 🌊

Owshen is the fanciest privacy solution ever built for Ethereum!

Join our Discord: [https://discord.gg/jMRRmANvf](https://discord.gg/jMRRmANvf)

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

## Abstract

Owshen is the very first implementation of an ***Anonymity Marketplace***. An Anonymity Marketplace is a market in which users metaphorically sell their identities, by mixing their identity into an anonymity pool. People who use this anonymity pool for the purpose of making their actions private (Buyers) will pay those who join the pool merely for increasing the size of the pool (Sellers). A large number of sellers will bring more buyers to the market (Given that the pool has got bigger thus more private), and a large number of buyers will bring more sellers (Since it gets very profitable to join as a seller). This whitepaper will discuss a practical implementation of such system using the help of zkSNARKs, and will analyze the dynamics of an Anonymity Marketplace.

Keywords: Privacy, Mixing, zkSNARK

## Introduction

Launched in 2009, Bitcoin was the first cryptocurrency to use a decentralized blockchain to record transactions. Users were identified by alphanumeric addresses, claiming to provide some degree of pseudonymity. While transaction details were visible, the real-world identities of users remained private unless they were voluntarily disclosed. The advance of chain analysis methods made it much easier to find the source and destination of transactions, breaking the privacy guarantees that Bitcoin claimed to have in the first place.

Right after Bitcoin, many cryptocurrency projects were built and introduced, bringing shinier privacy techniques on the table. The essence of many of those privacy techniques were essentially the same. They were trying to gather a lot of transactions together, and mix them into a single, hard to analyze, transaction. This is known as a CoinJoin in UTXO cryptocurrencies like Bitcoin.

A significant concern in private cryptocurrencies, is the size of their anonymity sets. An anonymity set refers to the group of users or participants from which a particular transaction can originate, making it challenging to determine the true source of that transaction. The larger the anonymity set, the more difficult it becomes to trace specific transactions back to their origin. On the other hand, if the anonymity set is small, it reduces the effectiveness of privacy measures and increases the risk of transaction tracing. No matter how good and secure the privacy protocol is, if there are very few people using it, there is no privacy. 

The anonymity set can have different meanings in different privacy-oriented currencies. In case of a CoinJoin, the anonymity set is the users who participate in the CoinJoin. In case of a TornadoCash-style mixing pool on Etheruem, the anonymity-set is the users who have interactions with the pool's smart contract, and in case of a whole private cryptocurrency like Monero, the anonymity set is the users who use Monero.

Knowing the importance of the anonymity sets in privacy-preserving cryptocurrencies, we tried to design a incentive model that encourages people to join an anonymity-set even when they do not need the privacy guarantees that the pool gives to them. We designed our model as a TornadoCash-style smart contract deployed on a blockchain which has massive number of users. Our implmentation is almost identical with TornadoCash with some key differences:

- There will be two kind of people using this platform:
    - **Buyers**: They want to anonymize their funds.
    - **Sellers**: They will help others to anonymize their funds.
- People deposit a fixed amount of tokens to the contract, plus an anonymizing fee. E.g. 1ETH + 0.1ETH (Anonymizing fee)
    - **NOTE:** Both **Buyers** and **Sellers** deposit same amount of ETH (1.1ETH) to the contract, so they are not distinguishable from the outside)
    - We will also keep track of the number of deposits: `active_deposits += 1;`
- The deposit commitments are saved on a sparse merkle tree besides their deposit date (Block number $b$) -> Leaf value: $H(b + H(s | 0))$:
    - Secret $s$
    - Commitment: $H(s | 0)$
    - Nullifier: $H(s | 1)$
    - Reward Nullifier: $H(s | 2 | m)$ (Where $m$ is the number of months past since the initial deposit)
- There will be two types of withdrawals:
    - **Withdrawal:** Prove that you know $s$ (Secret) where $H(s | 0)$ (Commitment to the secret) exists in the tree and its nullifier is $H(s | 1)$ which is not previously withdrawn, you will receive 1.0ETH. After revealing the nullifier, the [nullifier]th leaf of another sparse merkle tree will be flagged true.
        - The number of active deposits decreases on withdrawals: `active_deposits -= 1;`
    - **Reward Withdrawal:** Prove that you know $s$ (Secret) where $H(s | 0)$ (Commitment) exists in the tree, which its reward-nullifier is $H(s | 2 | m)$, and coinage is above a threshold, but $H(s | 1)$ (The nullifier of that secret) does not exist in nullifier sparse-merkle-tree (I.e. tokens are held and not withdrawn). Then you will receive `address(this).balance / active_deposits` worth of ETH. Notice that you can request a reward after every month, since a new reward-nullifier gets withdrawable for you per month you keep your funds on the contract. You can deposit once and have an steady stream of rewards which get withdrawable every month.

### Security analysis

 - Input transactions are all 1.1ETH, so people can't tell whether an input is a Buyer or a Seller
 - Majority of output transactions are 1.0ETH, so people can't tell whether an output is a Buyer's output or part of Seller's reward.
 - Some output transactions will be (0.1 + r)ETH, which means that output belongs to an Seller, but people won't know it corresponds to which input.

## Calculations

Here we provide calculation of Seller profits in case of different number of Buyers and Sellers. 

| Buyers |   Sellers   |  Input  | Buyers Output | Anonymizer Reward |
|--------|-------------|---------|---------------|-------------------|
|    0   |     100     |   110$  |      0$       |  110/100 = 1.1$   |
|    1   |      99     |   110$  |      1$       |   109/99 = 1.101$ |
|   50   |      50     |   110$  |     50$       |    60/50 = 1.2$   |
|   99   |       1     |   110$  |     99$       |     11/1 = 11.0$  |
|  100   |       0     |   110$  |    100$       |         N/A       |

The worst case is when all of the people participating in the protocol are sellers (I.e they are there to get revenue). In this case, they will get 0 reward and they will basically get their original money back.

In case the number of Buyers and Sellers are equal, the profit of a seller is approximately: $\frac{f}{1 + f}$ where $f$ is the anonymizing fee. As an example, if anonymizing fee is 10% of the value deposited, the seller's profit would be: $\frac{0.1}{1+0.1} \simeq  0.091$, around 9%.

And lastly, if the number of Buyers is much higher than the number of Sellers, there will be a big pool of rewards for Sellers to claim. Thus it's in their best interest to join the network as soon as possible and claim those rewards, leading to an increase in the size of the anonymity set, which will make the pool even more private.
