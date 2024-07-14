scoreboard players add base_index registers 1
execute store result storage memory:temp base_index int 1.0 run scoreboard players get base_index registers
data modify storage memory:stack frame append value {}

function my_datapack:main-body with storage memory:temp

function mcscript:pop_frame with storage memory:temp
