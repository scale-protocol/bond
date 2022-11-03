localmint:
	spl-token create-token --decimals 6 -- local_mint.json
	spl-token create-account -- local_mint.json
	spl-token mint 5Uzq44UgPkPNxG4E4m4m7F8fsnrHKc4jFvFuPapV4jN2 10000000000
initvault:
	cargo run -p cli -- init_vault
initmarket:
	cargo run -p cli -- init_market -p 'BTC/USD' -s 0.01 -y 'HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J' \
	-t 'CzZQBrJCLqjXRfMjRN3fhbxur2QYHUzkpaRwkWsiPqbz'
	cargo run -p cli -- init_market -p 'ETH/USD' -s 0.05 -y 'EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw' \
	-t '2ypeVyYnZaW2TNYXXTaZq9YhYvnqcjCiifW1C6n8b7Go'
	cargo run -p cli -- init_market -p 'SOL/USD' -s 0.05 -y 'J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix' \
	-t 'HgTtcbcmp5BeThax5AU8vg4VwK79qAvAKKFMs8txMLW6'
inituser:
	cargo run -p cli -- init_user
deposit:
	cargo run -p cli -- deposit -a 1000
investment:
	cargo run -p cli -- investment -p 'BTC/USD' -a 10000
	cargo run -p cli -- investment -p 'ETH/USD' -a 10000
	cargo run -p cli -- investment -p 'SOL/USD' -a 10000
divestment:
	cargo run -p cli -- divestment -p 'BTC/USD' -a 10000
	cargo run -p cli -- divestment -p 'ETH/USD' -a 5000
	cargo run -p cli -- divestment -p 'SOL/USD' -a 5000
openposition:
	cargo run -p cli -- open_position -p 'BTC/USD' -s 1.1 -l 20 -t 1 -d 1
closeposition:
	cargo run -p cli -- close_position -o 1
bot:
	cargo run -p cli -- bot