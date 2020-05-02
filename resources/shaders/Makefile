VERT=$(wildcard *.vert)
FRAG=$(wildcard *.frag)

all: $(VERT:=.spv) $(FRAG:=.spv)

clean:
	rm *.spv

%.vert.spv: %.vert
	glslc $< -o $@

%.frag.spv: %.frag
	glslc $< -o $@
