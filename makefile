all: export
export: fig1-export fig3-export evals-export

# NOTE: BIOS settings used:
# - no hyperthreading
# - `balanced` performance, even with A/C power.
#   - `performance` leads to throttling
# - laptop plugged in
evals:
	# Make sure there are no uncommited changes, and log commit ids.
	echo A*PA > evals/commit-ids
	git diff-index --quiet HEAD
	git rev-parse --short HEAD >> evals/commit-ids
	echo Edlib >> evals/commit-ids
	git -C ../edlib diff-index --quiet HEAD
	git -C ../edlib rev-parse --short HEAD >> evals/commit-ids
	echo WFA2 >> evals/commit-ids
	git -C ../wfa2 diff-index --quiet HEAD
	git -C ../wfa2 rev-parse --short HEAD >> evals/commit-ids
	# Build tools
	cargo build --release
	cargo build --release --example generate_dataset
	# Set CPU frequency
	sudo cpupower frequency-set -g performance
	sudo cpupower frequency-set -d 2.6GHz
	sudo cpupower frequency-set -u 2.6GHz
	# Run snakemake on 3 threads, with 3 jobs in parallel
	cd evals && \
	  taskset -c 0,2,4 snakemake -j 3 --rerun-incomplete \
	  table/scaling_n_N1e7.tsv \
	  table/tools_N1e7.tsv \
	  table/scaling_e_N1e6.tsv \

evals-export:
	cd evals && python3 ./figures.py
	rm -rf ../pairwise-aligner-paper/imgs/scaling/*
	cp evals/imgs/tools_*.pdf evals/imgs/scaling_e.pdf evals/imgs/scaling_n.pdf ../pairwise-aligner-paper/imgs/scaling/

fig1:
	cargo run --release --example fig1
	mogrify -format png imgs/fig1/*bmp

fig1-export: fig1
	rm -rf ../pairwise-aligner-paper/imgs/fig1/*
	mkdir -p ../pairwise-aligner-paper/imgs/fig1/
	cp imgs/fig1/*.png ../pairwise-aligner-paper/imgs/fig1/

# https://superuser.com/questions/1049606/reduce-generated-gif-size-using-ffmpeg
FILTER = -filter_complex "split[s0][s1];[s0]palettegen=max_colors=64[p];[s1][p]paletteuse=dither=bayer"

fig1-video:
	# mp4
	ffmpeg -y -framerate 1 -i imgs/fig1/1_ukkonen/%d.bmp imgs/fig1/1_ukkonen.mp4
	ffmpeg -y -framerate 10 -i imgs/fig1/2_dijkstra/%d.bmp imgs/fig1/2_dijkstra.mp4
	ffmpeg -y -framerate 10 -i imgs/fig1/3_diagonal-transition/%d.bmp imgs/fig1/3_diagonal_transition.mp4
	ffmpeg -y -framerate 20 -i imgs/fig1/4_dt-divide-and-conquer/%d.bmp imgs/fig1/4_dt-divide-and-conquer.mp4
	ffmpeg -y -framerate 60 -i imgs/fig1/5_astar-csh-pruning/%d.bmp imgs/fig1/5_astar-csh-pruning.mp4
	# gif
	ffmpeg -y -framerate 1 -i imgs/fig1/1_ukkonen/%d.bmp 				$(FILTER) imgs/fig1/1_ukkonen.gif
	ffmpeg -y -framerate 10 -i imgs/fig1/2_dijkstra/%d.bmp 				$(FILTER) imgs/fig1/2_dijkstra.gif
	ffmpeg -y -framerate 10 -i imgs/fig1/3_diagonal-transition/%d.bmp 	$(FILTER) imgs/fig1/3_diagonal_transition.gif
	ffmpeg -y -framerate 20 -i imgs/fig1/4_dt-divide-and-conquer/%d.bmp $(FILTER) imgs/fig1/4_dt-divide-and-conquer.gif
	ffmpeg -y -framerate 60 -i imgs/fig1/5_astar-csh-pruning/%d.bmp 	$(FILTER) imgs/fig1/5_astar-csh-pruning.gif

# Remove video source files
fig1-video-clean:
	rm -rf imgs/fig1/*/

fig3:
	cargo run --release --example fig3
	mogrify -format png imgs/fig3/*bmp

fig3-video:
	ffmpeg -y -framerate 20 -i imgs/fig3-video/%d.bmp imgs/fig3.mp4
	ffmpeg -y -framerate 20 -i imgs/fig3-video/%d.bmp $(FILTER) imgs/fig3.gif

fig3-export: fig3
	rm -rf ../pairwise-aligner-paper/imgs/fig3/*
	mkdir -p ../pairwise-aligner-paper/imgs/fig3/
	cp imgs/fig3/0.png ../pairwise-aligner-paper/imgs/fig3/start.png
	cp imgs/fig3/1.png ../pairwise-aligner-paper/imgs/fig3/end.png

fig3-video-clean:
	rm -rf imgs/fig3-video

fig-readme:
	cargo run --release --example fig-readme

fig-readme-video:
	ffmpeg -y -framerate 50 -i imgs/fig-readme/%d.bmp -vf fps=50 imgs/fig-readme.mp4
	ffmpeg -y -framerate 50 -i imgs/fig-readme/%d.bmp $(FILTER),fps=50 imgs/fig-readme.gif

.PHONY: all evals
