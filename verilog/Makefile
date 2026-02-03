# Sampo CPU Verilog Makefile
# Supports Icarus Verilog (iverilog) and Verilator

# Directories
RTL_DIR = rtl
TB_DIR = tb
PROG_DIR = programs
BUILD_DIR = build

# Source files
RTL_SRCS = $(RTL_DIR)/sampo_pkg.vh \
           $(RTL_DIR)/alu.v \
           $(RTL_DIR)/shifter.v \
           $(RTL_DIR)/regfile.v \
           $(RTL_DIR)/decode.v \
           $(RTL_DIR)/cpu.v \
           $(RTL_DIR)/ram.v \
           $(RTL_DIR)/uart.v \
           $(RTL_DIR)/soc.v

# Include path
VFLAGS = -I$(RTL_DIR)

# Default target
.PHONY: all
all: test

# Create build directory
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# ============================================================================
# Icarus Verilog Targets
# ============================================================================

# Compile ALU testbench
$(BUILD_DIR)/alu_tb: $(RTL_DIR)/alu.v $(TB_DIR)/alu_tb.v | $(BUILD_DIR)
	iverilog $(VFLAGS) -o $@ $(RTL_DIR)/alu.v $(TB_DIR)/alu_tb.v

# Compile register file testbench
$(BUILD_DIR)/regfile_tb: $(RTL_DIR)/regfile.v $(TB_DIR)/regfile_tb.v | $(BUILD_DIR)
	iverilog $(VFLAGS) -o $@ $(RTL_DIR)/regfile.v $(TB_DIR)/regfile_tb.v

# Compile main testbench
$(BUILD_DIR)/sampo_tb: $(RTL_SRCS) $(TB_DIR)/sampo_tb.v | $(BUILD_DIR)
	iverilog $(VFLAGS) -o $@ \
		$(RTL_DIR)/alu.v \
		$(RTL_DIR)/shifter.v \
		$(RTL_DIR)/regfile.v \
		$(RTL_DIR)/decode.v \
		$(RTL_DIR)/cpu.v \
		$(RTL_DIR)/ram.v \
		$(RTL_DIR)/uart.v \
		$(RTL_DIR)/soc.v \
		$(TB_DIR)/sampo_tb.v

# Run ALU tests
.PHONY: test-alu
test-alu: $(BUILD_DIR)/alu_tb
	cd $(BUILD_DIR) && ./alu_tb

# Run register file tests
.PHONY: test-regfile
test-regfile: $(BUILD_DIR)/regfile_tb
	cd $(BUILD_DIR) && ./regfile_tb

# Run main simulation
.PHONY: test
test: $(BUILD_DIR)/sampo_tb $(PROG_DIR)/hello.hex
	cd $(BUILD_DIR) && ./sampo_tb

# Run all unit tests
.PHONY: unit-tests
unit-tests: test-alu test-regfile

# ============================================================================
# Verilator Targets (optional, for faster simulation)
# ============================================================================

.PHONY: verilate
verilate: | $(BUILD_DIR)
	verilator --cc --exe --build -j 0 \
		$(VFLAGS) \
		--top-module soc \
		$(RTL_DIR)/alu.v \
		$(RTL_DIR)/shifter.v \
		$(RTL_DIR)/regfile.v \
		$(RTL_DIR)/decode.v \
		$(RTL_DIR)/cpu.v \
		$(RTL_DIR)/ram.v \
		$(RTL_DIR)/uart.v \
		$(RTL_DIR)/soc.v \
		--Mdir $(BUILD_DIR)/verilator

# ============================================================================
# Program Assembly
# ============================================================================

# Convert binary to Verilog hex format
$(PROG_DIR)/hello.hex: ../examples/hello.bin
	python3 bin2hex.py $< $@

# Assemble and convert any program
$(PROG_DIR)/%.hex: ../examples/%.bin
	python3 bin2hex.py $< $@

# Assemble source to binary (if needed)
../examples/%.bin: ../examples/%.s
	cd ../sasm && cargo run --release -- ../$< -o ../$@

# ============================================================================
# Utilities
# ============================================================================

# View waveforms (requires GTKWave)
.PHONY: wave
wave: $(BUILD_DIR)/sampo.vcd
	gtkwave $(BUILD_DIR)/sampo.vcd &

# Clean build artifacts
.PHONY: clean
clean:
	rm -rf $(BUILD_DIR)
	rm -f *.vcd

# Help
.PHONY: help
help:
	@echo "Sampo CPU Verilog Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  all          - Run main test (default)"
	@echo "  test         - Run main simulation with hello.hex"
	@echo "  test-alu     - Run ALU unit tests"
	@echo "  test-regfile - Run register file unit tests"
	@echo "  unit-tests   - Run all unit tests"
	@echo "  verilate     - Build with Verilator (faster simulation)"
	@echo "  wave         - View waveforms in GTKWave"
	@echo "  clean        - Remove build artifacts"
	@echo "  help         - Show this help"
	@echo ""
	@echo "Requirements:"
	@echo "  - Icarus Verilog (iverilog)"
	@echo "  - Rust/Cargo (for assembler)"
	@echo "  - GTKWave (optional, for waveforms)"
	@echo "  - Verilator (optional, for faster simulation)"
