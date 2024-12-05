import cv2
import numpy as np
import gradio as gr
import datetime
import uuid
import csv
import os
from typing import Dict, List

def get_model_type_dict(model_type: str):
    model_dir = {

    "sd":"./models/sd/",
    "lora":"./models/lora/",
    "segment":"./models/segment/",
    }

    model_dir = model_dir[model_type]
    model_dict = {"None":"None"}
    if model_type!="sd":
        for file in os.listdir(model_dir):
            name = file.split(".")[0]
            model_dict[name] = model_dir+file
    else:
        with open("./models/sd/models.txt", 'r') as f:
            for line in f.readlines():
                model_repo_id = line.strip()
                name = line.split('/')[1]
                model_dict[name] = model_repo_id 
    return model_dict
        
    
   

def append_to_csv(filename, model, lora, pos_prompt, neg_prompt, num_steps, guidance, seed, csv_file):
    # Data to be written to the CSV file
    data = [filename, model, lora, pos_prompt, neg_prompt, num_steps, guidance, seed]

    # Open the file in append mode
    with open(csv_file, 'a', newline='') as file:
        writer = csv.writer(file)
        # Write the data
        writer.writerow(data)
        
def generate_unique_filename(extension='.jpg'):
    # Get the current date and time
    now = datetime.datetime.now()
    # Format the date and time in a readable format
    formatted_date = now.strftime("%Y%m%d")
    # Generate a unique identifier
    unique_id = str(uuid.uuid4())[:8]
    # Combine all parts to form the filename
    filename = f"{formatted_date}_{unique_id}{extension}"
    return filename



def select_point(image, sel_pnt, point_type, event: gr.SelectData):
    colors = [(255,0,0), (0,255,0)]
    markers = [1,5]
    if point_type == 'foreground_point':
        sel_pnt.append((event.index,1))
    elif point_type == 'background_point':
        sel_pnt.append((event.index,0))
    for point, label in sel_pnt:
        cv2.drawMarker(image, point, colors[label], markerType=markers[label], markerSize=20, thickness=5)
    if image[..., 0][0,0] == image[..., 2][0, 0]:
        image = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)
    print(sel_pnt)
    return image if isinstance(image, np.ndarray) else np.array(image)
    
def undo_point(original, selected_point):
    if len(selected_point) != 0:
        selected_point.clear()
    return original
