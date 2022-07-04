all: fig1
export: fig1-export

fig1:
	cargo run --example fig1
	mogrify -format png imgs/fig1/*bmp


fig1-export: fig1
	rm -rf ../pairwise-aligner-paper/imgs/fig1
	cp -r imgs/fig1/*.png ../pairwise-aligner-paper/imgs
