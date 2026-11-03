# Run by Blender (scripts/mixamo_clips.lua): one Mixamo clip onto the CC0 animation library's skeleton, so the
# existing retargeters take it to every Tripo rig. World rotation of each bone relative to each rig's own T-pose;
# the pelvis follows at the leg-length ratio. Nothing but the motion comes across.
import sys
import bpy
from math import radians
from mathutils import Matrix, Quaternion

library_fbx, tpose_fbx, clip_fbx, out_fbx, first_kept, last_kept, yaw_degrees, wrist_degrees, return_after, hand_turn = sys.argv[sys.argv.index('--') + 1:]

PAIRS = [('pelvis', 'Hips'), ('spine_01', 'Spine'), ('spine_02', 'Spine1'), ('spine_03', 'Spine2'), ('neck_01', 'Neck'), ('Head', 'Head')]
for side, name in (('l', 'Left'), ('r', 'Right')):
    PAIRS += [(f'clavicle_{side}', f'{name}Shoulder'), (f'upperarm_{side}', f'{name}Arm'), (f'lowerarm_{side}', f'{name}ForeArm'), (f'hand_{side}', f'{name}Hand'),
              (f'thigh_{side}', f'{name}UpLeg'), (f'calf_{side}', f'{name}Leg'), (f'foot_{side}', f'{name}Foot'), (f'ball_{side}', f'{name}ToeBase')]
    for finger, label in (('thumb', 'Thumb'), ('index', 'Index'), ('middle', 'Middle'), ('ring', 'Ring'), ('pinky', 'Pinky')):
        PAIRS += [(f'{finger}_0{n}_{side}', f'{name}Hand{label}{n}') for n in (1, 2, 3)]


def imported(path):
    before = set(bpy.data.objects)
    bpy.ops.import_scene.fbx(filepath=path, ignore_leaf_bones=False)
    return next(o for o in bpy.data.objects if o not in before and o.type == 'ARMATURE')


def use(armature, action, frame):
    armature.animation_data_create()
    armature.animation_data.action = action
    if action.slots:
        armature.animation_data.action_slot = action.slots[0]
    bpy.context.scene.frame_set(frame)
    bpy.context.view_layer.update()


bpy.ops.wm.read_factory_settings(use_empty=True)
library = imported(library_fbx)
tpose_action = next(a for a in bpy.data.actions if a.name.endswith('|A_TPose'))
known = set(bpy.data.actions)
source_t = imported(tpose_fbx)
t_action = next(a for a in bpy.data.actions if a not in known)
known = set(bpy.data.actions)
source = imported(clip_fbx)
clip_action = next(a for a in bpy.data.actions if a not in known)

first, last = (int(round(v)) for v in clip_action.frame_range)
if last_kept:
    first, last = first + int(first_kept), min(last, first + int(last_kept))
bpy.context.scene.render.fps = 30

use(library, tpose_action, int(tpose_action.frame_range[0]))
library_t = {b: library.pose.bones[b].matrix.to_quaternion() for b, _ in PAIRS}
pelvis_t = library.pose.bones['pelvis'].matrix.translation.copy()
use(source_t, t_action, int(t_action.frame_range[0]))
source_t_world = {m: (source_t.matrix_world @ source_t.pose.bones['mixamorig:' + m].matrix).to_quaternion() for _, m in PAIRS}
hips_t = (source_t.matrix_world @ source_t.pose.bones['mixamorig:Hips'].matrix).translation.copy()
ratio = pelvis_t.z / hips_t.z
print(f'OM_MIXAMO frames {first}-{last} leg ratio {ratio:.5f}')

baked = bpy.data.actions.new(name='Armature|Armature|' + out_fbx.replace('\\', '/').split('/')[-1][:-4])
library.animation_data.action = baked
slot = baked.slots.new(id_type='OBJECT', name=library.name)
library.animation_data.action_slot = slot
for pb in library.pose.bones:
    pb.rotation_mode = 'QUATERNION'
    pb.matrix_basis = Matrix.Identity(4)

yaw = Matrix.Rotation(radians(float(yaw_degrees)), 4, 'Z')
order = sorted(PAIRS, key=lambda row: len(library.pose.bones[row[0]].parent_recursive))
frames = 0
return_after = int(return_after)
opening, held = {}, {}
for frame in range(first, last + 1):
    bpy.context.scene.frame_set(frame)
    bpy.context.view_layer.update()
    for bone, name in order:
        pb = library.pose.bones[bone]
        world = yaw @ source.matrix_world @ source.pose.bones['mixamorig:' + name].matrix
        rotation = world.to_quaternion() @ source_t_world[name].inverted() @ library_t[bone]
        if bone == 'hand_r' and float(wrist_degrees):
            at_bone = lambda part: (yaw @ source.matrix_world @ source.pose.bones['mixamorig:RightHand' + part].matrix).translation
            across = at_bone('Pinky1') - at_bone('Index1')
            along = at_bone('Middle1') - world.translation
            rotation = Quaternion(across.cross(along).normalized(), radians(float(wrist_degrees))) @ rotation
        if bone == 'hand_r' and float(hand_turn):
            rotation = Quaternion((0, 0, 1), radians(float(hand_turn))) @ rotation
        at = pb.matrix.translation.copy()
        if bone == 'pelvis':
            at = pelvis_t + (world.translation - hips_t) * ratio
        if frame == first:
            opening[bone] = (rotation.copy(), at.copy())
        if return_after and frame - first == return_after:
            held[bone] = (rotation.copy(), at.copy())
        if return_after and frame - first > return_after:
            t = (frame - first - return_after) / (last - first - return_after)
            t = t * t * (3 - 2 * t)
            rotation = held[bone][0].slerp(opening[bone][0], t)
            if bone == 'pelvis':
                at = held[bone][1].lerp(opening[bone][1], t)
        pb.matrix = Matrix.Translation(at) @ rotation.to_matrix().to_4x4()
        bpy.context.view_layer.update()
    for pb in library.pose.bones:
        pb.keyframe_insert('rotation_quaternion', frame=frame - first)
        pb.keyframe_insert('location', frame=frame - first)
    frames += 1
print(f'OM_MIXAMO baked {frames} frames')

for obj in list(bpy.data.objects):
    if obj not in (library,):
        bpy.data.objects.remove(obj)
bpy.context.scene.frame_start = 0
bpy.context.scene.frame_end = frames - 1
bpy.ops.object.select_all(action='DESELECT')
library.select_set(True)
bpy.context.view_layer.objects.active = library
bpy.ops.export_scene.fbx(filepath=out_fbx, use_selection=True, object_types={'ARMATURE'}, add_leaf_bones=False, bake_anim=True,
                         bake_anim_use_all_actions=False, bake_anim_use_nla_strips=False, bake_anim_force_startend_keying=True, bake_anim_simplify_factor=0.0)
print('OM_MIXAMO wrote', out_fbx)
