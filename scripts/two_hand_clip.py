# Run by Blender (scripts/mixamo_clips.lua): a clip with both hands on one weapon, on the library's skeleton.
# swing keeps the right arm's motion and puts the left hand on the haft; hold carries the weapon across the chest
# (for standing, running and casts without a swing of their own). The weapon runs along the fist's index-pinky axis,
# its head on the thumb side (flip in moba_tripo_hero_visuals), so both fists close on the same line.
import sys
import bpy
from math import acos, sqrt
from mathutils import Matrix, Quaternion, Vector

library_fbx, source, out_fbx, mode = sys.argv[sys.argv.index('--') + 1:]

SWING_HANDS_APART = 0.22
HOLD_HANDS_APART = 0.30
ARM_STRETCH = 0.995
# Right fist and weapon direction in the chest's frame (side, forward, up), metres from the chest.
HOLD_FIST = (0.20, 0.28, -0.32)
HOLD_HEAD = (-0.55, 0.15, 0.82)
HOLD_FINGERS = (-0.3, 0.5, 0.0)
ELBOW_HINT = {'r': (0.6, -0.1, -1.0), 'l': (-0.6, -0.1, -1.0)}

FINGERS = [f'{finger}_0{n}' for finger in ('thumb', 'index', 'middle', 'ring', 'pinky') for n in (1, 2, 3)]
UPPER = ['pelvis', 'spine_01', 'spine_02', 'spine_03', 'neck_01', 'Head']
LEGS = [f'{part}_{side}' for side in 'lr' for part in ('thigh', 'calf', 'foot', 'ball')]
ARMS = [f'{bone}_{side}' for side in 'lr' for bone in ['clavicle', 'upperarm', 'lowerarm', 'hand'] + FINGERS]
BONES = UPPER + LEGS + ARMS
SOLVED = {f'{bone}_{side}' for side in 'lr' for bone in ['upperarm', 'lowerarm', 'hand'] + FINGERS}
UP = Vector((0, 0, 1))


def imported(path):
    before = set(bpy.data.objects)
    known = set(bpy.data.actions)
    bpy.ops.import_scene.fbx(filepath=path, ignore_leaf_bones=False)
    armature = next(o for o in bpy.data.objects if o not in before and o.type == 'ARMATURE')
    return armature, [a for a in bpy.data.actions if a not in known]


def use(armature, action, frame):
    armature.animation_data_create()
    armature.animation_data.action = action
    if action.slots:
        armature.animation_data.action_slot = action.slots[0]
    bpy.context.scene.frame_set(frame)
    bpy.context.view_layer.update()


def world_pose(armature):
    return {b: armature.matrix_world @ armature.pose.bones[b].matrix for b in BONES}


bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.context.scene.render.fps = 30
library, library_actions = imported(library_fbx)
named = {a.name.split('|')[-1]: a for a in library_actions}
use(library, named['A_TPose'], int(named['A_TPose'].frame_range[0]))
t_pose = world_pose(library)
library_rest = {b: t_pose[b].to_quaternion() for b in BONES}

baked_dir = out_fbx.replace('\\', '/').rsplit('/', 1)[0]
grip_clip, grip_actions = imported(baked_dir + '/Mx_Axe_Smash.fbx')
use(grip_clip, grip_actions[0], int(grip_actions[0].frame_range[0]))
grip_pose = world_pose(grip_clip)
curl = {side: {f: grip_pose[f'hand_{side}'].to_quaternion().inverted() @ grip_pose[f'{f}_{side}'].to_quaternion() for f in FINGERS} for side in 'lr'}
bpy.data.objects.remove(grip_clip)

if source.startswith('library:'):
    armature, action = library, named[source[len('library:'):]]
else:
    armature, actions = imported(baked_dir + '/' + source + '.fbx')
    action = actions[0]
    bpy.data.objects.remove(library)
first, last = int(action.frame_range[0]), int(action.frame_range[1])
frames = []
for frame in range(first, last + 1):
    use(armature, action, frame)
    frames.append(world_pose(armature))


def basis(across, along):
    x = across.normalized()
    y = (along - x * along.dot(x)).normalized()
    return Matrix((x, y, x.cross(y))).transposed()


def hand_axes(pose, side):
    return pose[f'pinky_01_{side}'].translation - pose[f'index_01_{side}'].translation, pose[f'middle_01_{side}'].translation - pose[f'hand_{side}'].translation


rest_axes = {side: basis(*hand_axes(t_pose, side)) for side in 'lr'}
arm_lengths = {side: ((t_pose[f'lowerarm_{side}'].translation - t_pose[f'upperarm_{side}'].translation).length,
                      (t_pose[f'hand_{side}'].translation - t_pose[f'lowerarm_{side}'].translation).length) for side in 'lr'}
worst_short = 0.0


def solve_arm(side, shoulder, wrist, hint):
    # Two bones from the shoulder to the wrist, the elbow bending toward the hint; a wrist out of reach stops at the arm's end.
    global worst_short
    upper, lower = arm_lengths[side]
    reach = wrist - shoulder
    distance = reach.length
    limit = (upper + lower) * ARM_STRETCH
    if distance > limit:
        worst_short = max(worst_short, distance - limit)
        reach = reach * (limit / distance)
        distance = limit
    direction = reach.normalized()
    along = (upper * upper - lower * lower + distance * distance) / (2 * distance)
    height = sqrt(max(upper * upper - along * along, 0.0))
    out = (hint - direction * hint.dot(direction)).normalized()
    elbow = shoulder + direction * along + out * height
    return elbow, shoulder + reach


def along_to(bone, side_bone_from, side_bone_to, to_vector):
    rest = t_pose[side_bone_to].translation - t_pose[side_bone_from].translation
    return rest.rotation_difference(to_vector) @ library_rest[bone]


def in_chest(pose, values):
    side = pose['upperarm_r'].translation - pose['upperarm_l'].translation
    side.z = 0
    side.normalize()
    forward = UP.cross(side)
    return side, forward, side * values[0] + forward * values[1] + UP * values[2]


results = []
for pose in frames:
    rotation = {}
    if mode == 'hold':
        side, forward, fist_offset = in_chest(pose, HOLD_FIST)
        chest = pose['spine_03'].translation
        right_wrist = chest + fist_offset
        _, _, head = in_chest(pose, HOLD_HEAD)
        _, _, fingers = in_chest(pose, HOLD_FINGERS)
        head.normalize()
        left_wrist = right_wrist + head * HOLD_HANDS_APART
        across, along = -head, fingers
    else:
        right_wrist = pose['hand_r'].translation
        across, along = hand_axes(pose, 'r')
        across = across.normalized()
        left_wrist = right_wrist + across * SWING_HANDS_APART
    side_now = pose['upperarm_r'].translation - pose['upperarm_l'].translation
    side_now.z = 0
    side_now.normalize()
    forward_now = UP.cross(side_now)
    for side, wrist in (('r', right_wrist), ('l', left_wrist)):
        if side == 'r' and mode != 'hold':
            continue
        x, y, z = ELBOW_HINT[side]
        hint = side_now * x + forward_now * y + UP * z
        elbow, reached = solve_arm(side, pose[f'upperarm_{side}'].translation, wrist, hint)
        shoulder = pose[f'upperarm_{side}'].translation
        rotation[f'upperarm_{side}'] = along_to(f'upperarm_{side}', f'upperarm_{side}', f'lowerarm_{side}', elbow - shoulder)
        rotation[f'lowerarm_{side}'] = along_to(f'lowerarm_{side}', f'lowerarm_{side}', f'hand_{side}', reached - elbow)
    for side in 'lr':
        if side == 'r' and mode != 'hold':
            hand = pose['hand_r'].to_quaternion()
        else:
            turn = (basis(across, along) @ rest_axes[side].inverted()).to_quaternion()
            hand = turn @ library_rest[f'hand_{side}']
            rotation[f'hand_{side}'] = hand
        for finger in FINGERS:
            rotation[f'{finger}_{side}'] = hand @ curl[side][finger]
    results.append(rotation)

baked = bpy.data.actions.new(name='Armature|Armature|' + out_fbx.replace('\\', '/').split('/')[-1][:-4])
armature.animation_data.action = baked
slot = baked.slots.new(id_type='OBJECT', name=armature.name)
armature.animation_data.action_slot = slot
for pb in armature.pose.bones:
    pb.rotation_mode = 'QUATERNION'
    pb.matrix_basis = Matrix.Identity(4)
bpy.context.view_layer.update()
order = sorted(BONES, key=lambda b: len(armature.pose.bones[b].parent_recursive))
to_armature = armature.matrix_world.inverted()
for index, (pose, rotation) in enumerate(zip(frames, results)):
    for bone in order:
        pb = armature.pose.bones[bone]
        if bone == 'pelvis' or bone not in SOLVED and bone not in rotation:
            world = pose[bone]
        else:
            at = (armature.matrix_world @ pb.matrix).translation
            world = Matrix.Translation(at) @ (rotation[bone] if bone in rotation else pose[bone].to_quaternion()).to_matrix().to_4x4()
        if bone != 'pelvis' and bone not in rotation:
            at = (armature.matrix_world @ pb.matrix).translation
            world = Matrix.Translation(at) @ pose[bone].to_quaternion().to_matrix().to_4x4()
        pb.matrix = to_armature @ world
        bpy.context.view_layer.update()
    for pb in armature.pose.bones:
        pb.keyframe_insert('rotation_quaternion', frame=index)
        pb.keyframe_insert('location', frame=index)
print(f'OM_TWO_HANDS {mode} {source}: {len(frames)} frames, off hand short of the haft by at most {worst_short:.2f} m')

for obj in list(bpy.data.objects):
    if obj is not armature:
        bpy.data.objects.remove(obj)
bpy.context.scene.frame_start = 0
bpy.context.scene.frame_end = len(frames) - 1
bpy.ops.object.select_all(action='DESELECT')
armature.select_set(True)
bpy.context.view_layer.objects.active = armature
bpy.ops.export_scene.fbx(filepath=out_fbx, use_selection=True, object_types={'ARMATURE'}, add_leaf_bones=False, bake_anim=True,
                         bake_anim_use_all_actions=False, bake_anim_use_nla_strips=False, bake_anim_force_startend_keying=True, bake_anim_simplify_factor=0.0)
print('OM_TWO_HANDS wrote', out_fbx)
