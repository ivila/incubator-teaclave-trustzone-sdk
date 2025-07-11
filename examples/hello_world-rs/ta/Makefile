# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

UUID ?= $(shell cat "../uuid.txt")

TARGET ?= aarch64-unknown-linux-gnu
CROSS_COMPILE ?= aarch64-linux-gnu-
OBJCOPY := $(CROSS_COMPILE)objcopy
# Configure the linker to use GCC, which works on both cross-compilation and ARM machines
LINKER_CFG := target.$(TARGET).linker=\"$(CROSS_COMPILE)gcc\"

TA_SIGN_KEY ?= $(TA_DEV_KIT_DIR)/keys/default_ta.pem
SIGN := $(TA_DEV_KIT_DIR)/scripts/sign_encrypt.py
OUT_DIR := $(CURDIR)/target/$(TARGET)/release

BUILDER ?= $(if $(STD),xargo,cargo)

all: ta strip sign

ta:
	@$(BUILDER) build --target $(TARGET) --release --config $(LINKER_CFG)

strip: ta
	@$(OBJCOPY) --strip-unneeded $(OUT_DIR)/ta $(OUT_DIR)/stripped_ta

sign: strip
	@$(SIGN) --uuid $(UUID) --key $(TA_SIGN_KEY) --in $(OUT_DIR)/stripped_ta --out $(OUT_DIR)/$(UUID).ta
	@echo "SIGN =>  ${UUID}"

emulate: all
	@sync_to_emulator --ta $(OUT_DIR)/$(UUID).ta

clean:
	@cargo clean
