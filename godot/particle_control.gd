extends GPUParticles2D

func _ready():
	emitting = false

func _process(_delta):
	var ball = get_parent()
	if not ball:
		return
	# 瞄准/拖拽中 (freeze=true) → 不发射
	# 飞行中 (freeze=false, 速度>0) → 发射
	# 碰地停止 (freeze=false, 速度≈0) → 不发射
	emitting = not ball.freeze and ball.linear_velocity.length() > 10.0
