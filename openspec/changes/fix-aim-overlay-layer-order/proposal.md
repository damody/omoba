# Correct aim and communication overlay paint layers

Shared production overlay reports base+1 even though cast guidance text paints at base+2 and communication center text at base+5. This violates the Slate child paint contract and can let child text escape above later siblings. Propagate the actual highest used layer through all early returns.
