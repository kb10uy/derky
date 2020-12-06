SHADER_DIR := derky-d3d11/shaders
SHADERS := \
    geometry \
	screen
INCLUDES := \
	common.hlsli

# -----------------------------------------------------------------------------
SHADERS_QUALIFIED = $(addprefix $(SHADER_DIR)/,$(SHADERS))
INCLUDES_QUALIFIED = $(addprefix $(SHADER_DIR)/,$(INCLUDES))

.PHONY: all

all: $(addsuffix .vso,$(SHADERS_QUALIFIED)) $(addsuffix .pso,$(SHADERS_QUALIFIED))

%.vso: %.hlsl $(INCLUDES_QUALIFIED)
	fxc.exe	/T vs_5_0 /E vertex_main /Fo $@ $<

%.pso: %.hlsl $(INCLUDES_QUALIFIED)
	fxc.exe	/T ps_5_0 /E pixel_main /Fo $@ $<
