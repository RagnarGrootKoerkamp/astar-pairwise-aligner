all: fig1
export: fig1-export scaling-export

fig1:
	cargo run --example fig1
	mogrify -format png imgs/fig1/*bmp


fig1-export: fig1
	rm -rf ../pairwise-aligner-paper/imgs/fig1
	cp -r imgs/fig1/*.png ../pairwise-aligner-paper/imgs

scaling-export:
	cp evals/imgs/tools_*.pdf evals/imgs/scaling_e.pdf evals/imgs/scaling_n.pdf ../pairwise-aligner-paper/imgs/scaling/
