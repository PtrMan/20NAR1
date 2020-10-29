pathTo20Nar = "/home/r0b3/dev/rust/20mlish6"
# load module from path for python 3.5+
# from https://stackoverflow.com/a/67692/388614
import importlib.util
spec = importlib.util.spec_from_file_location("module.Binding", pathTo20Nar+"/Binding.py")
Binding = importlib.util.module_from_spec(spec)
spec.loader.exec_module(Binding)

b = Binding.Binding(pathTo20Nar) # instantiate python binding for 20NAR1

import math
import numpy as np
import pybullet as p
import time
import pybullet_data
physicsClient = p.connect(p.GUI)#or p.DIRECT for non-graphical version
p.setAdditionalSearchPath(pybullet_data.getDataPath()) #optionally
p.setGravity(0,0,-10)
planeId = p.loadURDF("plane.urdf")
cubeStartPos = [0,0,1]
cubeStartOrientation = p.getQuaternionFromEuler([0,0,0])
robotId = p.loadURDF("r2d2.urdf",cubeStartPos, cubeStartOrientation)



mass = 1.0

sphereRadius = 0.05
colSphereId = p.createCollisionShape(p.GEOM_SPHERE, radius=sphereRadius)
colBoxId = p.createCollisionShape(p.GEOM_BOX,
                                  halfExtents=[sphereRadius, sphereRadius, sphereRadius])

mass = 1
visualShapeId = -1

basePosition = [1.0, 0.0, 3.0]
baseOrientation = [0, 0, 0, 1]

phyObjUid = p.createMultiBody(mass, colSphereId, visualShapeId, basePosition,
                                      baseOrientation)

p.changeDynamics(phyObjUid,
                -1,
                spinningFriction=0.001,
                rollingFriction=0.001,
                linearDamping=0.0)


# register ops
#b.i("!por NOP ^left")
#b.i("!por NOP ^right")
#b.i("!por NOP ^forward")
#b.i("!por NOP ^backward")


# op to set distance to 2
b.i("!por NOP ^setDist2")
# op to set distance to 4
b.i("!por NOP ^setDist4")


# set motor velocity for testing
maxForce = 100.0
targetVel = 15.0


# interpret robot command and set controls for physics
def roboCmd(code):
    jointRightIdxs = [2, 3] # right front and back wheel
    jointLeftIdxs = [6, 7] # left front and back wheel

    jointIdxsPos = []
    jointIdxsNeg = []
    
    if code == "l":
        jointIdxsPos = jointLeftIdxs
        jointIdxsNeg = jointRightIdxs
    elif code == "r":
        jointIdxsPos = jointRightIdxs
        jointIdxsNeg = jointLeftIdxs
    elif code == "f" or code == "f2": # forward
        jointIdxsPos = jointRightIdxs + jointLeftIdxs
        jointIdxsNeg = []
    elif code == "b" or code == "b2": # backward
        jointIdxsPos = []
        jointIdxsNeg = jointRightIdxs + jointLeftIdxs
    
    thisMaxForce = maxForce
    thisTargetVel = targetVel
    if code == "f" or code == "f2":
        thisMaxForce = maxForce * 0.05 # slow
        thisTargetVel = targetVel * 0.5
    if code == "b" or code == "b2":
        thisMaxForce = maxForce * 0.05 # slow
        thisTargetVel = targetVel * 0.5
        

    for iJointIdx in jointIdxsPos:
        p.setJointMotorControl2(bodyUniqueId=robotId,
            jointIndex=iJointIdx,
            controlMode=p.VELOCITY_CONTROL,
            targetVelocity = thisTargetVel,
            force = thisMaxForce)
    
    for iJointIdx in jointIdxsNeg:
        p.setJointMotorControl2(bodyUniqueId=robotId,
            jointIndex=iJointIdx,
            controlMode=p.VELOCITY_CONTROL,
            targetVelocity = -thisTargetVel,
            force = thisMaxForce)

#roboCmd("l") # for testing

for idx in range(15):
    print(idx)
    print(
        p.getJointInfo(
            bodyUniqueId = robotId,
            jointIndex = idx))


def normalize(vec):
    vec2 = np.array(vec[:])
    len2 = math.sqrt(vec2[0]**2.0+vec2[1]**2.0+vec2[2]**2.0)
    vec2 /= len2
    return vec2
    
def dot(a,b):
    return a[0]*b[0]+ a[1]*b[1]+ a[2]*b[2]


distToTargetGoal = 2.0 # goal distance to target

oldState = ""

for i in range(100000000):
    p.stepSimulation()
    p.stepSimulation()
    time.sleep(1./120.)

    robotPos, robotOrn = p.getBasePositionAndOrientation(robotId)
    targetPos, targetOrn = p.getBasePositionAndOrientation(phyObjUid)

    r2d2ornEuler = p.getEulerFromQuaternion(robotOrn)
    yaw = r2d2ornEuler[2]
    #yaw += 3.141*2.0 # correct by rotating 90 degree
    yaw -= 3.141*0.5

    # this is a "basic rotation around Z
    # see https://en.wikipedia.org/wiki/Rotation_matrix "Basic Rotations"
    robotDir = np.array([math.cos(-yaw), -math.sin(-yaw), 0])
    # rotated by 90 degrees because we care only about side
    robotDirZ90 = np.array([math.cos(-(yaw-3.141*0.5)), -math.sin(-(yaw-3.141*0.5)), 0])
    diffRobotToTarget = np.array([(robotPos[0]-targetPos[0]),(robotPos[1]-targetPos[1]),(robotPos[2]-targetPos[2])])
    normalizedDiffRobotToTarget = normalize(diffRobotToTarget)
    # compute dot product to get direction dot vector
    sideDot = dot(robotDirZ90, normalizedDiffRobotToTarget)
    dirDot = dot(robotDir, normalizedDiffRobotToTarget)
    dirDotNotNormalized = dot(robotDir, diffRobotToTarget)

    if False: # deebug dir and dist etc
        print("[d] dirDot"+str(dirDot))
        print("[d] robo dir"+str(robotDir))
        print("[d] diff "+str(diffRobotToTarget[0])+"," +str(diffRobotToTarget[1])+","+str(diffRobotToTarget[2]))
        print("[d] dirDotNotNormalized "+str(dirDotNotNormalized))

    distToTarget = dirDotNotNormalized # distance to target is the dot product

    #if i > 100:
    #    break
    state2 = "" # more detailed state
    if i % 1 == 0: # send state to NAR?
        state = ""

        if dirDot > 0.0: # is robot pointing to target?
            if np.abs(sideDot) < 0.3:
                state = "c"
                state2 = "c"
            elif sideDot > 0.8:
                state = "l2"
                state2 = "l"
            elif sideDot > 0.0:
                state = "l"
                state2 = "l"
            elif sideDot < -0.8:
                state = "r2"
                state2 = "r"
            else:
                state = "r"
                state2 = "r"
        else:
            state = "back"
            state2 = "back"

        
        #print(state)
        if state != oldState:
            #b.i(state+". :|:") # send current state
            #print(state)
            oldState = state
    
    #print(state2)

    distToTargetDiff = None

    # hardcoded low level control
    if True:
        if state2 == "back":
            roboCmd("l")
        if state2 == "r":
            roboCmd("l")
        elif state2 == "l":
            roboCmd("r")
        elif state2 == "c":
            pass # do nothing

            
            # we can now adjust the distance to the target
            
            #distToTargetGoal = 2.0 # goal distance to target
            distToTargetDiff = distToTargetGoal - distToTarget 
            #print(distToTargetDiff)
            if np.abs(distToTargetDiff) < 0.3:
                pass # don't do anything to not topple robot over
            elif distToTargetDiff > 0.0:
                if distToTargetDiff > 0.8:
                    roboCmd("f2") # soft forward to not topple robot over
                else:
                    roboCmd("f")
            else:
                if distToTargetDiff > -0.8:
                    roboCmd("b")
                else:
                    roboCmd("b2")
            
            
            
    if i % 40 == 0:
        if distToTarget == None:
            pass
        elif distToTarget < 2.0:
            b.i("db2. :|:")
        elif distToTarget > 2.0 and distToTarget < 3.0:
            b.i("d2. :|:")
        elif distToTarget > 3.0 and distToTarget < 4.0:
            b.i("d3. :|:")
        elif distToTarget > 4.0 and distToTarget < 5.0:
            b.i("da4. :|:")
    
    if i % 40*2 == 0: # refresh goal
        
        b.i("d2! :|:")
        pass

    # procedural step for NAR
    if i % 40 == 0:
        b.sp()

    while True:
        narLine = b.tryRead() # try to read from binding to NAR
        if narLine == None:
            #print("NOTHING", flush=True)
            break
        if narLine: # was something returned?
            trimmedNarLine = narLine.rstrip()
            
            if trimmedNarLine[0] != "!": # is it not a command?
                print("[d] NAR returned:"+trimmedNarLine, flush=True) # for debugging
                pass
            
            if len(trimmedNarLine) > 0 and trimmedNarLine[-1] == "!": # is a execution?
                if trimmedNarLine.find("^left") != -1: # left op
                    print("OP left", flush=True)

                    roboCmd("l")

                elif trimmedNarLine.find("^right") != -1: # right op
                    print("OP right", flush=True)

                    roboCmd("r")
                elif trimmedNarLine.find("^setDist2") != -1:
                    distToTargetGoal = 2.0
                elif trimmedNarLine.find("^setDist4") != -1:
                    distToTargetGoal = 4.0


cubePos, cubeOrn = p.getBasePositionAndOrientation(boxId)
print(cubePos,cubeOrn)

#experiment
#cuid = pybullet.createCollisionShape(pybullet.GEOM_BOX, halfExtents = [1, 1, 1])
#mass= 0 #static box
#pybullet.createMultiBody(mass,cuid)





p.disconnect()
