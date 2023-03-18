all:

# ========== PAPER FIGURES ==========
#
paper: fig-intro fig-layers fig-limitations fig-comparison

paper-export:
	rm -rf ../astarpa-paper/imgs/{intro/*,layers/*,limitations/*,comparison/*}
	rsync -a imgs/paper/intro/*png ../astarpa-paper/imgs/intro/
	rsync -a imgs/paper/layers/*png ../astarpa-paper/imgs/layers/
	rsync -a imgs/paper/limitations/*png ../astarpa-paper/imgs/limitations/
	rsync -a imgs/paper/comparison/*png ../astarpa-paper/imgs/comparison/

fig-intro:
	rm -rf imgs/paper/intro/*
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

# ========== README VIDEOS ==========
readme: readme-layers readme-intro
readme-layers:
	cargo run --features example --release --example readme-layers
	ffmpeg -y -framerate 20 -i imgs/readme/layers/%d.bmp $(FILTER) imgs/readme/layers.gif
	gifsicle -O3 --batch imgs/readme/layers.gif
	rm -rf imgs/readme/layers/

# https://superuser.com/questions/1049606/reduce-generated-gif-size-using-ffmpeg
readme-intro:
	FILTER = -filter_complex "tpad=stop_mode=clone:stop_duration=2[t];[t]split[s0][s1];[s0]palettegen=max_colors=64[p];[s1][p]paletteuse=dither=bayer"
	cargo run --features example --release --example readme-videos
	ffmpeg -y -framerate 1 -i imgs/readme/1_ukkonen/%d.bmp 				$(FILTER) imgs/readme/1_ukkonen.gif
	ffmpeg -y -framerate 10 -i imgs/readme/2_dijkstra/%d.bmp 				$(FILTER) imgs/readme/2_dijkstra.gif
	ffmpeg -y -framerate 10 -i imgs/readme/3_diagonal-transition/%d.bmp 	$(FILTER) imgs/readme/3_diagonal_transition.gif
	ffmpeg -y -framerate 20 -i imgs/readme/4_dt-divide-and-conquer/%d.bmp $(FILTER) imgs/readme/4_dt-divide-and-conquer.gif
	ffmpeg -y -framerate 2 -i imgs/readme/5_astarpa/%d.bmp 	$(FILTER) imgs/readme/5_astarpa.gif
	gifsicle -O3 --batch imgs/readme/*.gif
	rm -rf imgs/readme/*/


# ========== BLOGSPOSTS IMAGES ==========
path-tracing:
	rm imgs/path-tracing/*
	cargo run --release --example path-tracing
	cargo run --release --example path-tracing-affine
	mogrify -format png imgs/path-tracing/*bmp
	rm imga/path-tracing/*bmp
path-tracing-export:
	cp imgs/path-tracing/*png ../../research/posts/linear-memory-wfa/
