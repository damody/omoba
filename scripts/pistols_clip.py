# Run by Blender (scripts/mixamo_clips.lua): the library has no two-pistol motion, so this builds one on its own skeleton.
# The one-pistol aim's right arm is mirrored onto the left, the body is turned to the unit's front and the guns levelled;
# the clip raises both guns to the aim, then both recoil together at the impact (impact_seconds in moba_tripo_hero_visuals).
import sys
import bpy
from math import atan2, degrees, radians, sin, pi
from mathutils import Matrix, Quaternion, Vector

library_fbx, out_fbx = sys.argv[sys.argv.index('--') + 1:]

FRAMES = 18
IMPACT_FRAME = 6
RECOIL_PEAK_FRAME = 9
LOWERED_DEGREES = -30.0
RECOIL_ARM_DEGREES = 10.0
RECOIL_HAND_DEGREES = 22.0
# The aim holds one pistol in both hands: open the arms so each gun is ahead of its own shoulder.
OPEN_ARMS_DEGREES = 16.0

ARM = ['clavicle', 'upperarm', 'lowerarm', 'hand'] + [f'{finger}_0{n}' for finger in ('thumb', 'index', 'middle', 'ring', 'pinky') for n in (1, 2, 3)]
SPINE = ['spine_01', 'spine_02', 'spine_03', 'neck_01', 'Head']
LEGS = [f'{part}_{side}' for side in 'lr' for part in ('thigh', 'calf', 'foot', 'ball')]
LEFT_ARM = {f'{bone}_l' for bone in ARM}
RAISED = {f'{bone}_{side}' for side in 'lr' for bone in ARM if bone != 'clavicle'}
HAND = {f'{bone}_{side}' for side in 'lr' for bone in ARM if bone not in ('clavicle', 'upperarm', 'lowerarm')}
BONES = ['pelvis'] + SPINE + LEGS + [f'{bone}_{side}' for side in 'lr' for bone in ARM]

bpy.ops.wm.read_factory_settings(use_empty=True)
before = set(bpy.data.objects)
bpy.ops.import_scene.fbx(filepath=library_fbx, ignore_leaf_bones=False)
library = next(o for o in bpy.data.objects if o not in before and o.type == 'ARMATURE')
bpy.context.scene.render.fps = 30


def action_named(tail):
    return next(a for a in bpy.data.actions if a.name.endswith('|' + tail))


def use(action):
    library.animation_data_create()
    library.animation_data.action = action
    if action.slots:
        library.animation_data.action_slot = action.slots[0]
    bpy.context.scene.frame_set(int(action.frame_range[0]))
    bpy.context.view_layer.update()


def pose():
    return {b: library.pose.bones[b].matrix.copy() for b in BONES}


use(action_named('A_TPose'))
t_pose = pose()
use(action_named('Pistol_Aim_Neutral'))
aim = pose()

aim_rotation = {b: aim[b].to_quaternion() for b in BONES}


def turn_about_up(left, right):
    # From the line's direction in T-pose: the library's armature space is the world turned half round.
    across, at_rest = aim[right].translation - aim[left].translation, t_pose[right].translation - t_pose[left].translation
    return (degrees(atan2(across.y, across.x) - atan2(at_rest.y, at_rest.x)) + 180) % 360 - 180


pelvis_turn = turn_about_up('thigh_l', 'thigh_r')
chest_turn = turn_about_up('upperarm_l', 'upperarm_r')
body = Quaternion((0, 0, 1), radians(-pelvis_turn))
chest = Quaternion((0, 0, 1), radians(pelvis_turn - chest_turn))
spine_share = {'spine_01': 1 / 3, 'spine_02': 2 / 3}
print(f'OM_PISTOLS pelvis {pelvis_turn:+.1f} chest {chest_turn:+.1f}')

# Both arms straight ahead of a chest facing the front: the aim's right arm with its chest's turn taken out, and its mirror image.
MIRROR = Matrix.Diagonal((-1.0, 1.0, 1.0)).to_3x3()
straight = Quaternion((0, 0, 1), radians(-chest_turn))
for bone in LEFT_ARM:
    right = bone[:-1] + 'r'
    aim_rotation[right] = straight @ aim[right].to_quaternion()
    turn = (aim_rotation[right].to_matrix() @ t_pose[right].to_3x3().inverted()).normalized()
    aim_rotation[bone] = ((MIRROR @ turn @ MIRROR) @ t_pose[bone].to_3x3()).to_quaternion()

finger = aim['middle_01_r'].translation - aim['hand_r'].translation
front = library.matrix_world.to_3x3().inverted() @ Vector((0, 1, 0))
level = (straight @ finger).rotation_difference(front)
level_left = (MIRROR @ level.to_matrix() @ MIRROR).to_quaternion()
across_axis = front.cross(Vector((0, 0, 1)))

baked = bpy.data.actions.new(name='Armature|Armature|Mx_Pistols_Fire')
library.animation_data.action = baked
slot = baked.slots.new(id_type='OBJECT', name=library.name)
library.animation_data.action_slot = slot
for pb in library.pose.bones:
    pb.rotation_mode = 'QUATERNION'
    pb.matrix_basis = Matrix.Identity(4)
bpy.context.view_layer.update()


def smooth(t):
    t = min(max(t, 0.0), 1.0)
    return t * t * (3 - 2 * t)


order = sorted(BONES, key=lambda b: len(library.pose.bones[b].parent_recursive))
for frame in range(FRAMES):
    raised = LOWERED_DEGREES * (1 - smooth(frame / IMPACT_FRAME))
    if frame < IMPACT_FRAME:
        recoil = 0.0
    elif frame < RECOIL_PEAK_FRAME:
        recoil = sin((frame - IMPACT_FRAME) / (RECOIL_PEAK_FRAME - IMPACT_FRAME) * pi / 2)
    else:
        recoil = 1 - smooth((frame - RECOIL_PEAK_FRAME) / (FRAMES - 1 - RECOIL_PEAK_FRAME))
    arm_pitch = Quaternion(across_axis, radians(raised + recoil * RECOIL_ARM_DEGREES))
    hand_pitch = Quaternion(across_axis, radians(recoil * RECOIL_HAND_DEGREES))
    for bone in order:
        pb = library.pose.bones[bone]
        arm = bone.startswith('clavicle') or bone in RAISED
        rotation = aim_rotation[bone] if arm else body @ aim_rotation[bone]
        if bone in spine_share:
            rotation = Quaternion((0, 0, 1), radians(pelvis_turn - chest_turn) * spine_share[bone]) @ rotation
        elif bone in ('spine_03', 'neck_01', 'Head'):
            rotation = chest @ rotation
        if bone in HAND:
            rotation = (level_left if bone in LEFT_ARM else level) @ rotation
        if bone in RAISED and bone not in HAND:
            rotation = Quaternion((0, 0, 1), radians(OPEN_ARMS_DEGREES if bone in LEFT_ARM else -OPEN_ARMS_DEGREES)) @ rotation
        if bone in RAISED:
            rotation = arm_pitch @ rotation
        if bone in HAND:
            rotation = hand_pitch @ rotation
        at = pb.matrix.translation.copy()
        if bone == 'pelvis':
            at = body @ aim[bone].translation
        pb.matrix = Matrix.Translation(at) @ rotation.to_matrix().to_4x4()
        bpy.context.view_layer.update()
    for pb in library.pose.bones:
        pb.keyframe_insert('rotation_quaternion', frame=frame)
        pb.keyframe_insert('location', frame=frame)

for obj in list(bpy.data.objects):
    if obj is not library:
        bpy.data.objects.remove(obj)
bpy.context.scene.frame_start = 0
bpy.context.scene.frame_end = FRAMES - 1
bpy.ops.object.select_all(action='DESELECT')
library.select_set(True)
bpy.context.view_layer.objects.active = library
bpy.ops.export_scene.fbx(filepath=out_fbx, use_selection=True, object_types={'ARMATURE'}, add_leaf_bones=False, bake_anim=True,
                         bake_anim_use_all_actions=False, bake_anim_use_nla_strips=False, bake_anim_force_startend_keying=True, bake_anim_simplify_factor=0.0)
print('OM_PISTOLS wrote', out_fbx)
