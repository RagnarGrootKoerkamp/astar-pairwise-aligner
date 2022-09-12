all:

# ========== SHORTHANDS ==========

# Generate figures 1 and 3.
figures: fig1 fig3 fig8 results

# Shorthands below are mostly for private use.

# Copy generated images and results to the paper.
export: fig1-export fig3-export results-export
# Generate videos for the figures above, for the readme.
videos: fig1-video fig3-video fig-readme-video
# Remove generated images for videos
videos-clean: fig1-video-clean fig3-video-clean fig-readme-video-clean

# ========== WFA & EDLIB SETUP ==========

# Clone WFA2-lib and build using makefile
wfa:
	cd .. && git clone https://github.com/smarco/WFA2-lib.git wfa2
	cd ../wfa2 && make

# Clone fork of Edlib and build using meson
edlib:
	cd .. && git clone https://github.com/RagnarGrootKoerkamp/edlib.git
	cd ../edlib && make

# ========== EVALS ==========

cpu-freq:
	@echo -e "Sudo is needed to call 'cpupower frequency-set' to pin the cpu frequency.\n\
Feel free to remove the dependency on the 'cpu-freq' rule from the makefile if you don't want this."
	sudo cpupower frequency-set -g performance
	sudo cpupower frequency-set -d 2.6GHz
	sudo cpupower frequency-set -u 2.6GHz

# NOTE: BIOS settings used:
# - no hyperthreading
# - `balanced` performance, even with A/C power.
#   - `performance` leads to throttling
# - laptop plugged in
evals: cpu-freq
	# Make sure there are no uncommited changes, and log commit ids.
	git diff-index --quiet HEAD
	echo A*PA > evals/commit-ids
	git rev-parse --short HEAD >> evals/commit-ids
	echo Edlib >> evals/commit-ids
	git -C ../edlib diff-index --quiet HEAD
	git -C ../edlib rev-parse --short HEAD >> evals/commit-ids
	echo WFA2 >> evals/commit-ids
	git -C ../wfa2 diff-index --quiet HEAD
	git -C ../wfa2 rev-parse --short HEAD >> evals/commit-ids
	# Build tools
	cargo build --no-default-features --release
	cargo build --no-default-features --release --example generate_dataset
	# Run snakemake on 3 threads, with 3 jobs in parallel.
	# The first rule `all` is executed automatically.
	cd evals && \
	  nice -n -20 \
	  taskset -c 0,2,4 \
	    snakemake -j 3 -f --rerun-incomplete \
	      table/scaling_e.tsv \
	      table/scaling_n.tsv \
	      table/tools.tsv \
	      table/human_ont-ul.tsv \
	      table/human_chm13.tsv \

results:
	cd evals && python3 ./results.py

results-export:
	rm -f ../pairwise-aligner-paper/imgs/fig4/*
	rm -f ../pairwise-aligner-paper/imgs/fig6/*
	cp evals/results/tools_*.pdf \
      ../pairwise-aligner-paper/imgs/fig4/
	cp evals/results/scaling_*.pdf \
      ../pairwise-aligner-paper/imgs/fig5/
	cp evals/results/expanded_*.pdf \
      ../pairwise-aligner-paper/imgs/fig6/
	cp evals/results/table* ../pairwise-aligner-paper/data/
	cp evals/results/speedup ../pairwise-aligner-paper/data/speedup
	cp evals/results/human* ../pairwise-aligner-paper/imgs/fig7/
	cp evals/results/stats_* ../pairwise-aligner-paper/data/
	cp evals/results/prune_fraction_* ../pairwise-aligner-paper/data/

# ========== IMAGES ==========

fig1:
	cargo run --features sdl2 --release --example fig1
	mogrify -format png imgs/fig1/*bmp

fig1-export: fig1
	rm -rf ../pairwise-aligner-paper/imgs/fig1/*
	mkdir -p ../pairwise-aligner-paper/imgs/fig1/
	cp imgs/fig1/*.png \
      ../pairwise-aligner-paper/imgs/fig1/

fig3:
	cargo run --features sdl2_ttf --release --example fig3
	mogrify -format png imgs/fig3/*bmp

fig3-export: fig3
	rm -rf ../pairwise-aligner-paper/imgs/fig3/*
	mkdir -p ../pairwise-aligner-paper/imgs/fig3/
	cp imgs/fig3/0.png ../pairwise-aligner-paper/imgs/fig3/start.png
	cp imgs/fig3/1.png ../pairwise-aligner-paper/imgs/fig3/end.png

fig8:
	cargo run --release --example fig8
	mogrify -format png imgs/fig8/*bmp

fig8-export: fig8
	rm -rf ../pairwise-aligner-paper/imgs/fig8/*
	mkdir -p ../pairwise-aligner-paper/imgs/fig8/
	cp imgs/fig8/*png ../pairwise-aligner-paper/imgs/fig8/

# ========== VIDEOS ==========

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

fig3-video:
	ffmpeg -y -framerate 20 -i imgs/fig3-video/%d.bmp imgs/fig3.mp4
	ffmpeg -y -framerate 20 -i imgs/fig3-video/%d.bmp $(FILTER) imgs/fig3.gif

fig3-video-clean:
	rm -rf imgs/fig3-video

fig-readme-video:
	cargo run --features sdl2_ttf --release --example fig-readme
	ffmpeg -y -framerate 50 -i imgs/fig-readme/%d.bmp -vf fps=50 imgs/fig-readme.mp4
	ffmpeg -y -framerate 50 -i imgs/fig-readme/%d.bmp $(FILTER),fps=50 imgs/fig-readme.gif

fig-readme-video-clean:
	rm -rf imgs/fig-readme

# ========== BLOGSPOSTS IMAGES ==========
path-tracing:
	rm imgs/path-tracing/*
	cargo run --features sdl2 --release --example path-tracing
	cargo run --features sdl2 --release --example path-tracing-affine
path-tracing-export:
	mogrify -format png imgs/path-tracing/*bmp
	cp imgs/path-tracing/*png ../../research/posts/linear-memory-wfa/

# ========== FLAMEGRAPHS ==========

flamegraphs: cpu-freq
	mkdir -p imgs/flamegraphs/
	cargo flamegraph -o imgs/flamegraphs/0.05.svg --bin astar-pairwise-aligner -- -n 10000000 -e 0.05 -k 15 -r 1 -a sh
	cargo flamegraph -o imgs/flamegraphs/0.15.svg --bin astar-pairwise-aligner -- -n 10000000 -e 0.15 -k 15 -r 2 -a csh

# ========== CONFIG ==========

.PHONY: all evals
