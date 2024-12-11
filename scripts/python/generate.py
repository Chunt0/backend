import torch
from diffusers import AutoencoderKL, DiffusionPipeline
from tqdm import gui
import utils
import argparse

def load_sdxl():
    # SDXL has a watermark, this is a hack to remove it
    #lora_model_dict = utils.get_model_type_dict("lora")
    class NoWatermark:
        def apply_watermark(self, img):
            return img 
            
    model_checkpoint = "RunDiffusion/Juggernaut-X-v10"
    lora_checkpoint_1 = "/home/chunt/Pictures/dataset/mifella/lora/model/m~f-collage-v2.safetensors"
    lora_checkpoint_2 = "/home/chunt/Pictures/dataset/alvin-baltrop/lora/alvin_baltrop_v2/model/b9av-photograph-v2.safetensors"
    
    # Load Tuned Base model and refiner
    pipe = DiffusionPipeline.from_pretrained(
        model_checkpoint, 
        torch_dtype=torch.float16,
        )
    #pipe.load_lora_weights(lora_checkpoint_1, weight_name="m~f-collage-v2.safetensors", adapter_name="mifella")
    #pipe.load_lora_weights(lora_checkpoint_2, weight_name="b9av-photograph-v2.safetensors", adapter_name="baltrop")
    #pipe.set_adapters(["mifella", "baltrop"], adapter_weights=[.75,1])
    pipe.watermark = NoWatermark()
    pipe.safety_checker = lambda images, **kwargs: (images, [False] * len(images))
    pipe.to("cuda")

    return pipe
    
def generate_sdxl(model, pos_prompt, neg_prompt, guidance, height, width, num_steps = 20, num_images = 1, seed = -1):
    generator = None if seed == -1 else torch.Generator("cuda").manual_seed(seed)
    images = model(
        prompt=pos_prompt,
        negative_prompt=neg_prompt,
        height=height,
        width=width,
        num_inference_steps=num_steps,
        num_images_per_prompt=num_images,
        guidance_scale=guidance,
        #output='latent',
        generator=generator,
    ).images
    for image in images:
        #filename = utils.generate_unique_filename()
        image.save(f"/home/chunt/Code/ppl-systems/backend/scripts/python/output/temp.png")

    return images[0]
    

def run_txt2img(pos_prompt, 
                neg_prompt, 
                guidance, 
                height, width,
                ):
    #num_images = num_images if num_images <= 10 else 10
    model = load_sdxl()
    return generate_sdxl(model, pos_prompt, neg_prompt, guidance, height, width)


def main():
    parser = argparse.ArgumentParser(description="Process generation parameters.")
    parser.add_argument('--pos_prompt', type=str, help='Positive prompt')
    parser.add_argument('--neg_prompt', type=str, help='Negative prompt')
    parser.add_argument('--prompt_strength', type=float, help='Prompt strength')
    parser.add_argument('--batch_size', type=int, help='Batch size')
    parser.add_argument('--height', type=int, help='Height of image in pixels')
    parser.add_argument('--width', type=int, help='Width of image in pixels')
    parser.add_argument('--loras', type=str, help='Comma-separated list of loras')
    parser.add_argument('--output', type=str, help='Output file path')

    args = parser.parse_args()
    
    pos_prompt = args.pos_prompt
    neg_prompt = args.neg_prompt
    guidance = 7.5 * args.prompt_strength
    height = args.height
    width = args.width

    run_txt2img(pos_prompt, neg_prompt, guidance, height, width)
    
if __name__ == "__main__":
    main()

