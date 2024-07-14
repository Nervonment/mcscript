scoreboard players set r0 registers 114
scoreboard players set r1 registers 514
scoreboard players operation r2 registers = r0 registers
scoreboard players operation r2 registers *= r1 registers
scoreboard players set r3 registers 19
scoreboard players set r4 registers 198
scoreboard players set r5 registers 10
scoreboard players operation r6 registers = r4 registers
scoreboard players operation r6 registers -= r5 registers
scoreboard players operation r7 registers = r3 registers
scoreboard players operation r7 registers *= r6 registers
scoreboard players operation r8 registers = r2 registers
scoreboard players operation r8 registers -= r7 registers
$execute store result storage memory:stack frame[$(base_index)].x@1 int 1.0 run scoreboard players get r8 registers
$execute store result score r0 registers run data get storage memory:stack frame[$(base_index)].x@1
scoreboard players set r1 registers 1
scoreboard players operation r2 registers = r0 registers
scoreboard players operation r2 registers += r1 registers
$execute store result storage memory:stack frame[$(base_index)].x@1 int 1.0 run scoreboard players get r2 registers
$execute store result score r0 registers run data get storage memory:stack frame[$(base_index)].x@1
scoreboard players operation return_value registers = r0 registers
