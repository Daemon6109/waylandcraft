package dev.evvie.waylandcraft.gui;

import org.joml.Matrix3x2fStack;

import dev.evvie.waylandcraft.WaylandCraft;
import dev.evvie.waylandcraft.bridge.WLCSurface;
import dev.evvie.waylandcraft.render.WindowFramebuffer;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;

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
		
		int x = 10;
		int y = 10;
		WLCSurface[] surfaces = WaylandCraft.instance.bridge.getAllSurfaces();
		for(WLCSurface surface : surfaces) {
			if(surface.getParent() != null) continue;
			
			WindowFramebuffer buf = surface.getFramebuffer();
			if(buf != null && buf.isValid()) {
				context.blit(buf.getTextureLocation(), x, y, x + buf.getWidth(), y + buf.getHeight(), 0, 1, 0, 1);
				x += buf.getWidth();
			}
		}
		
		poseStack.popMatrix();
		
		super.extractRenderState(context, mouseX, mouseY, partialTicks);
	}
	
	@Override
	public void extractBackground(GuiGraphicsExtractor guiGraphics, int i, int j, float f) {
	}
	
}
