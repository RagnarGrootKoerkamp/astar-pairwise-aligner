all: fig1

fig1:
	cargo run --example fig1
	mogrify -format png imgs/fig1/*bmp

