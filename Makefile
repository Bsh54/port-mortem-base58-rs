RUST_BIN := target/release/base58
ORACLE   := oracle-go/oracle

.PHONY: all build test fuzz bench oracle clean

all: build

build:
	cargo build --release --locked

test:
	cargo test --release

$(ORACLE):
	cd oracle-go && go mod tidy && go build -o oracle .

oracle: $(ORACLE)

fuzz: build oracle
	python3 fuzz/differential.py --rust $(RUST_BIN) --go $(ORACLE) --seconds 60 --seed 1

bench: build oracle
	python3 bench/run.py --rust $(RUST_BIN) --go $(ORACLE)

clean:
	cargo clean
	rm -f oracle-go/oracle oracle-go/oracle.exe
