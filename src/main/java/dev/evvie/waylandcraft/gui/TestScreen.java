package dev.evvie.waylandcraft.gui;

import org.joml.Matrix3x2fStack;

import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.textures.FilterMode;

import dev.evvie.waylandcraft.WaylandCraft;
import dev.evvie.waylandcraft.bridge.WLCSurface;
import dev.evvie.waylandcraft.render.BufferTexture;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;
import net.minecraft.util.ARGB;

public class TestScreen extends Screen {
	
	public TestScreen() {
		super(Component.literal("test"));
	}
	
	@Override
	public boolean isPauseScreen() {
		return false;
	}
	
	@Override
	public void extractRenderState(GuiGraphicsExtractor context, int mouseX, int mouseY, float partialTicks) {
		super.extractBlurredBackground(context);
		
		float guiScale = (float) Minecraft.getInstance().getWindow().getGuiScale();
		Matrix3x2fStack poseStack = context.pose();
		poseStack.pushMatrix();
		poseStack.scale(1 / guiScale, 1 / guiScale);
		
		context.fill(0, 0, 0, 0, 0); // force blurred background to render
		
		WLCSurface root = null;
		WLCSurface[] surfaces = WaylandCraft.instance.bridge.getAllSurfaces();
		for(WLCSurface surface : surfaces) {
			if(surface.getParent() == null) {
				root = surface;
			}
		}
		
		WLCSurface[] tree;
		if(root != null && (tree = root.getDrawTree()) != null) {
			for(WLCSurface surface : tree) {
				int x = 10 + surface.xSubpos;
				int y = 10 + surface.ySubpos;
				int w = surface.width();
				int h = surface.height();
				
				context.outline(x, y, w, h, ARGB.color(255, 255, 255));
				
				BufferTexture buffer;
				if((buffer = surface.getBuffer()) != null) {
					context.blit(buffer.getTextureView(), RenderSystem.getSamplerCache().getRepeat(FilterMode.NEAREST), x, y, x + w, y + h, 0, 1, 0, 1);
				}
			}
		}
		
//		int fbWidth = Minecraft.getInstance().getWindow().getWidth();
//		int x = 0;
//		int y = 0;
//		int hmax = 0;
//		for(WLCSurface surface : WaylandCraft.instance.bridge.getAllSurfaces()) {
//			int w = surface.width();
//			int h = surface.height();
//			
//			context.outline(x, y, w, h, ARGB.color(255, 255, 255));
//			
//			BufferTexture buffer;
//			if((buffer = surface.getBuffer()) != null) {
//				context.blit(buffer.getTextureView(), RenderSystem.getSamplerCache().getRepeat(FilterMode.NEAREST), x, y, x + w, y + h, 0, 1, 0, 1);
//			}
//			
//			if(h > hmax) hmax = h;
//			if(x + w > fbWidth) {
//				y += hmax;
//				x = 0;
//				hmax = 0;
//			}
//			else {
//				x += w;
//			}
//		}
		
		poseStack.popMatrix();
		
		super.extractRenderState(context, mouseX, mouseY, partialTicks);
	}
	
	@Override
	public void extractBackground(GuiGraphicsExtractor guiGraphics, int i, int j, float f) {
	}
	
}
