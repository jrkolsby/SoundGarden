import os
import sys
import cv2
import math
import numpy as np
from PIL import Image, ImageFont, ImageDraw
from shapely import geometry, ops
from skimage import morphology, filters, img_as_ubyte
from octree import OctNode, Window, Line

# FONT RENDERING

SIZE = 512
HOUGH_THRESH = 3
HOUGH_MIN_LEN = 20
HOUGH_MAX_GAP = 7
OUTPUT_DIR = 'output'
#CHARSET = "人攴彡ミワヮムトエイィ厂二工田冂冖丿ノ艸匚爪巛州儿冫山彳乙丶ᄽᄾᄿᅴᆚᅐᅀᄁᆺᆽᅕ〥〤〻〲〱々〆〇〰ᄼQWERTYUIOPASDFGHJKLZXCVBNMqwertyuiopasdfghjklzxcvbnm,<.>/?;:'\"~`1234567890-=!@#$%^&*()_+[]{}\\|▗▖▄▝▐▞▟▘▚▌▙▀▜▛█━┃┅┇┉┋┏┓┗┛┣┫┳┻╋╏"
CHARSET = "QWERTYUIOPASDFGHJKLZXCVBNMqwertyuiopasdfghjklzxcvbnm,<.>/?;:'\"~`1234567890-=!@#$%^&*()_+[]{}\\|▗▖▄▝▐▞▟▘▚▌▙▀▜▛█━┃┅┇┉┋┏┓┗┛┣┫┳┻╋╏"

# Initialize unicode font

input_font = sys.argv[1]
input_file = os.path.basename(input_font)
input_name = input_file.split('.')[0]

output_svg = '{0}/{1}.svg'.format(OUTPUT_DIR, input_name)
output_xml = '{0}/{1}.xml'.format(OUTPUT_DIR, input_name)

print(input_name)

font = ImageFont.truetype(input_font, SIZE) # Load font

'''
    We want to save this data in a way that can be quickly written into memory
    at startup, preferably such that the consumer doesn't need to generate the 
    octree on its own. So we will store the octree order in a seperate table
    which can be loaded to generate the octree far more efficiently
'''

# Initialize SVG to display results

svg = open(output_svg, 'w+')
svg.write('<svg width="{0}" height="{1}" xmlns="http://www.w3.org/2000/svg">'
        .format(SIZE, SIZE * len(CHARSET)))
svg.write('<style> .small { font: italic 5px sans-serif; } </style>')

dim = SIZE // 2
root = OctNode(Window(dim, dim, dim, dim))

for char in CHARSET:

    print(char, end="", flush=True)

    char_img = Image.new("L", (SIZE, SIZE), 0) #Initialize black bg
    draw = ImageDraw.Draw(char_img)
    draw.text((0, 0), char, font=font, fill=(255)) # Draw white char

    img = np.array(char_img) # Convert to numpy array

    # SKELETONIZATION and PATH DETECTION (perform hough transform)

    img = cv2.blur(img, (29,29)) # erode

    try:
        img = img > filters.threshold_otsu(img) # Convert to 1 bit image
        img = morphology.skeletonize(img)
        img = img_as_ubyte(img)
    except:
        continue

    edges = cv2.Canny(img, 50, 150, apertureSize = 3)
    lines = cv2.HoughLinesP(edges, 1, np.pi/180, 
            threshold = HOUGH_THRESH,
            minLineLength = HOUGH_MIN_LEN,
            maxLineGap = HOUGH_MAX_GAP)

    # SIMPLIFY LINE TOPOLOGY WITH MERGE

    line_string = [] 

    if lines is None:
        continue

    for line in lines:
        for x1, y1, x2, y2 in line:
            line_string.append(geometry.LineString([[x1,y1], [x2,y2]]))

    multi_line = geometry.MultiLineString(line_string)

    merged_line = ops.linemerge(multi_line)

    if merged_line.geometryType() is not 'MultiLineString':
        continue

    portion = int(50 / len(merged_line))

    for line in merged_line:
        [[x1, y1], [x2, y2]] = line.coords

        dx = x2 - x1
        dy = y2 - y1

        theta = 90 if dx == 0 else (180 * math.atan(dy / dx)) / np.pi
        length = math.sqrt(dx**2 + dy**2)
        mid_x = x1 + (dx / 2)
        mid_y = y1 + (dy / 2)

        theta = int(theta)
        length = int(length)
        mid_x = int(mid_x)
        mid_y = int(mid_y)

        theta_norm = int(SIZE * (theta + 90) / 180)
        root.insert(Line(mid_x, mid_y, theta_norm, length, portion, char))

        svg.write('<path stroke="red" stroke-width="1" d="M {0} {1} {2} {3} "/>'
                .format(int(x1), int(y1), int(x2), int(y2)))

        svg.write('<text class="small" x="{0}" y="{1}">({2},{3},{4})</text>'
                .format(mid_x, mid_y, mid_x, mid_y, theta_norm))

svg.write('"</svg>')
svg.close()

output = open(output_xml, 'w+')
root.write(output, 0)

print("\nWrote ['{0}', '{1}']".format(output_svg, output_xml))