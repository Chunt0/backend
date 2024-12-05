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
            
    model_checkpoint = "stabilityai/stable-diffusion-xl-base-1.0"
    #lora_checkpoint = lora_model_dict[lora_model_type]
    
    # Get VAE for SDXL this is hardcoded
    vae = AutoencoderKL.from_pretrained("madebyollin/sdxl-vae-fp16-fix", torch_dtype=torch.float16)
    # Load Tuned Base model and refiner
    pipe = DiffusionPipeline.from_pretrained(
        model_checkpoint, 
        torch_dtype=torch.float16,
        vae=vae,
        )
    #if lora_model_type != "None":
    #    pipe.load_lora_weights(lora_checkpoint)
    #    pipe.fuse_lora()
    pipe.watermark = NoWatermark()
    pipe.safety_checker = lambda images, **kwargs: (images, [False] * len(images))
    pipe.to("cuda")

    return pipe
    
def generate_sdxl(model, pos_prompt, neg_prompt, guidance, num_steps = 20, num_images = 1, seed = -1):
    generator = None if seed == -1 else torch.Generator("cuda").manual_seed(seed)
    images = model(
        prompt=pos_prompt,
        negative_prompt=neg_prompt,
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
                ):
    #num_images = num_images if num_images <= 10 else 10
    model = load_sdxl()
    return generate_sdxl(model, pos_prompt, neg_prompt, guidance)


def main():
    parser = argparse.ArgumentParser(description="Process generation parameters.")
    parser.add_argument('--pos_prompt', type=str, help='Positive prompt')
    parser.add_argument('--neg_prompt', type=str, help='Negative prompt')
    parser.add_argument('--prompt_strength', type=float, help='Prompt strength')
    parser.add_argument('--batch_size', type=int, help='Batch size')
    parser.add_argument('--size', type=str, help='Size in WxH format')
    parser.add_argument('--loras', type=str, help='Comma-separated list of loras')
    parser.add_argument('--output', type=str, help='Output file path')

    args = parser.parse_args()
    
    pos_prompt = args.pos_prompt
    neg_prompt = args.neg_prompt
    guidance = 7.5 * args.prompt_strength
    run_txt2img(pos_prompt, neg_prompt, guidance)
    
if __name__ == "__main__":
    main()

