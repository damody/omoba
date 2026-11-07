# Run by Blender (scripts/mixamo_clips.lua): the library has no two-pistol motion, so this builds one on its own skeleton.
# The one-pistol aim's right arm is mirrored onto the left, the body is turned to the unit's front and the guns levelled;
# fire_r and fire_l raise one gun and fire it at the impact (impact_seconds in moba_tripo_hero_visuals), the attack alternating
# them (AOmAnimatedUnitActor::SelectNativeAnimationSlot); volley turns on the spot, a shot to the front, right, back and left.
# ready keeps a source clip's body with the guns held low and forward; the shots start and end in that hold, so standing,
# running and firing follow each other without the arms jumping (nothing blends between clips at runtime).
import sys
import bpy
from math import atan2, degrees, radians, sin, pi
from mathutils import Matrix, Quaternion, Vector

library_fbx, *source, out_fbx, mode = sys.argv[sys.argv.index('--') + 1:]

FRAMES = 30 if mode == 'volley' else 24
# Frame of each shot, its hand and the body's turn when it fires (degrees about up: clockwise from above, the order front, right, back, left).
SHOTS = {'fire_r': [(6, 'r', 0.0)], 'fire_l': [(6, 'l', 0.0)], 'ready': [(6, 'r', 0.0)],
         'volley': [(6, 'r', 0.0), (12, 'l', -90.0), (18, 'r', -180.0), (24, 'l', -270.0)]}[mode]
RECOIL_RISE_FRAMES, RECOIL_FALL_FRAMES = (2, 3) if mode == 'volley' else (3, 5)
# The frame the guns start back down to the hold.
LOWER_FROM = 26 if mode == 'volley' else 15
# The hold: how far each part of the arm drops from the aim.
READY_DEGREES = {'upperarm': -65.0, 'lowerarm': -25.0, 'hand': -12.0}
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


use(action_named('Idle_Loop'))
standing = pose()


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

baked = bpy.data.actions.new(name='Armature|Armature|' + out_fbx.replace('\\', '/').split('/')[-1][:-4])
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


def blend(a, b, t):
    return a.slerp(-b if a.dot(b) < 0 else b, t)


def aimed(bone):
    # The bone's direction in the aim, with the body at the front and the guns level.
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
    return rotation


def held(bone):
    part = 'hand' if bone in HAND else bone[:-2]
    return Quaternion(across_axis, radians(READY_DEGREES[part])) @ aimed(bone) if part in READY_DEGREES else aimed(bone)


def aiming(frame, side):
    # How far this side's arm has come up from the hold; a single shot only raises the arm that fires.
    if mode != 'volley' and side != SHOTS[0][1]:
        return 0.0
    return min(smooth(frame / (SHOTS[0][0] - 1)), 1 - smooth((frame - LOWER_FROM) / (FRAMES - 1 - LOWER_FROM)))


order = sorted(BONES, key=lambda b: len(library.pose.bones[b].parent_recursive))

def recoil_at(frame, side):
    recoil = 0.0
    for shot, hand, _ in SHOTS:
        if hand != side or frame < shot:
            continue
        if frame < shot + RECOIL_RISE_FRAMES:
            recoil = max(recoil, sin((frame - shot) / RECOIL_RISE_FRAMES * pi / 2))
        else:
            recoil = max(recoil, 1 - smooth((frame - shot - RECOIL_RISE_FRAMES) / RECOIL_FALL_FRAMES))
    return recoil


def turn_at(frame):
    # Holds each shot's direction, then swings to the next; the last swing comes round to the front again.
    stops = [(shot, turn) for shot, _, turn in SHOTS] + [(FRAMES - 1, -360.0)]
    for (start, turn), (end, next_turn) in zip(stops, stops[1:]):
        if frame <= end:
            return turn + (next_turn - turn) * smooth((frame - start) / (end - start))
    return 0.0


if mode == 'ready':
    action = action_named(source[0][len('library:'):])
    first, last = int(action.frame_range[0]), int(action.frame_range[1])
    frames = []
    library.animation_data.action = action
    library.animation_data.action_slot = action.slots[0]
    for frame in range(first, last + 1):
        bpy.context.scene.frame_set(frame)
        bpy.context.view_layer.update()
        frames.append(pose())
    library.animation_data.action = baked
    library.animation_data.action_slot = slot
    for pb in library.pose.bones:
        pb.matrix_basis = Matrix.Identity(4)
    bpy.context.view_layer.update()
    FRAMES = len(frames)

for frame in range(FRAMES):
    spin = Quaternion((0, 0, 1), radians(turn_at(frame) if mode == 'volley' else 0.0))
    for bone in order:
        pb = library.pose.bones[bone]
        arm = bone.startswith('clavicle') or bone in RAISED
        at = pb.matrix.translation.copy()
        if mode == 'ready':
            rotation = held(bone) if arm else frames[frame][bone].to_quaternion()
            if bone == 'pelvis':
                at = frames[frame][bone].translation
        else:
            side = 'l' if bone in LEFT_ARM else 'r'
            recoil = recoil_at(frame, side)
            rotation = aimed(bone)
            if bone in RAISED:
                rotation = Quaternion(across_axis, radians(recoil * RECOIL_ARM_DEGREES)) @ rotation
            if bone in HAND:
                rotation = Quaternion(across_axis, radians(recoil * RECOIL_HAND_DEGREES)) @ rotation
            rotation = spin @ rotation
            # The arms come up from the hold and go back to it, the body from standing and back.
            weight = aiming(frame, side if arm else SHOTS[0][1])
            rotation = blend(held(bone) if arm else standing[bone].to_quaternion(), rotation, weight)
            if bone == 'pelvis':
                at = standing[bone].translation.lerp(spin @ body @ aim[bone].translation, weight)
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
