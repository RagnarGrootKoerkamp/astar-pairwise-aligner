all:

# ========== SHORTHANDS ==========
# Generate videos for the figures above, for the readme.
videos: fig-intro-video fig_layers-video fig-readme-video
# Remove generated images for videos
videos-clean: fig-intro-video-clean fig_layers-video-clean fig-readme-video-clean

# ========== PAPER FIGURES ==========

fig-intro:
	rm -rf imgs/paper/intro/*/*.bmp
	rm -rf imgs/paper/intro/*.bmp
	rm -rf imgs/paper/intro/*.png
	cargo run --features example --release --example fig-intro
	mogrify -format png imgs/paper/intro/*bmp
	rm imgs/paper/intro/*bmp

fig-layers:
	rm -f imgs/paper/layers/*
	cargo run --features example --release --example fig-layers
	mogrify -format png imgs/paper/layers/*bmp
	rm imgs/paper/layers/*bmp

fig-limitations:
	cargo run --features example --release --example fig-limitations
	mogrify -format png imgs/paper/limitations/*bmp
	rm imgs/paper/limitations/*bmp

fig-comparison:
	cargo run --features example --release --example fig-comparison
	mogrify -format png imgs/paper/comparison/*bmp
	rm imgs/paper/comparison/*bmp

paper-figs: fig-intro fig-layers fig-limitations

paper-export-only:
	rm -rf ../astarpa-paper/imgs/{intro/*,layers/*,limitations/*,comparison/*}
	rsync -a imgs/paper/intro/*png ../astarpa-paper/imgs/intro/
	rsync -a imgs/paper/layers/*png ../astarpa-paper/imgs/layers/
	rsync -a imgs/paper/limitations/*png ../astarpa-paper/imgs/limitations/
	rsync -a imgs/paper/comparison/*png ../astarpa-paper/imgs/comparison/

paper-export: paper-figs paper-export-only

# ========== VIDEOS ==========

# https://superuser.com/questions/1049606/reduce-generated-gif-size-using-ffmpeg
FILTER = -filter_complex "tpad=stop_mode=clone:stop_duration=2[t];[t]split[s0][s1];[s0]palettegen=max_colors=64[p];[s1][p]paletteuse=dither=bayer"

fig-intro-video:
	# mp4
	ffmpeg -y -framerate 1 -i imgs/paper/intro/1_ukkonen/%d.bmp imgs/paper/intro/1_ukkonen.mp4
	ffmpeg -y -framerate 10 -i imgs/paper/intro/2_dijkstra/%d.bmp imgs/paper/intro/2_dijkstra.mp4
	ffmpeg -y -framerate 10 -i imgs/paper/intro/3_diagonal-transition/%d.bmp imgs/paper/intro/3_diagonal_transition.mp4
	ffmpeg -y -framerate 20 -i imgs/paper/intro/4_dt-divide-and-conquer/%d.bmp imgs/paper/intro/4_dt-divide-and-conquer.mp4
	ffmpeg -y -framerate 2 -i imgs/paper/intro/5_astarpa/%d.bmp imgs/paper/intro/5_astarpa.mp4
	# gif
	ffmpeg -y -framerate 1 -i imgs/paper/intro/1_ukkonen/%d.bmp 				$(FILTER) imgs/paper/intro/1_ukkonen.gif
	ffmpeg -y -framerate 10 -i imgs/paper/intro/2_dijkstra/%d.bmp 				$(FILTER) imgs/paper/intro/2_dijkstra.gif
	ffmpeg -y -framerate 10 -i imgs/paper/intro/3_diagonal-transition/%d.bmp 	$(FILTER) imgs/paper/intro/3_diagonal_transition.gif
	ffmpeg -y -framerate 20 -i imgs/paper/intro/4_dt-divide-and-conquer/%d.bmp $(FILTER) imgs/paper/intro/4_dt-divide-and-conquer.gif
	ffmpeg -y -framerate 2 -i imgs/paper/intro/5_astarpa/%d.bmp 	$(FILTER) imgs/paper/intro/5_astarpa.gif

# Remove video source files
fig-intro-video-clean:
	rm -rf imgs/paper/intro/*/*.bmp

fig_layers-video:
	ffmpeg -y -framerate 20 -i imgs/fig_layers-video/%d.bmp imgs/fig_layers.mp4
	ffmpeg -y -framerate 20 -i imgs/fig_layers-video/%d.bmp $(FILTER) imgs/fig_layers.gif

fig_layers-video-clean:
	rm -rf imgs/fig_layers-video

fig-readme-video:
	cargo run --features vis --release --example fig-readme
	ffmpeg -y -framerate 50 -i imgs/fig-readme/%d.bmp -vf fps=50 imgs/fig-readme.mp4
	ffmpeg -y -framerate 50 -i imgs/fig-readme/%d.bmp $(FILTER),fps=50 imgs/fig-readme.gif

fig-readme-video-clean:
	rm -rf imgs/fig-readme

# ========== BLOGSPOSTS IMAGES ==========
path-tracing:
	rm imgs/path-tracing/*
	cargo run --features vis --release --example path-tracing
	cargo run --features vis --release --example path-tracing-affine
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
