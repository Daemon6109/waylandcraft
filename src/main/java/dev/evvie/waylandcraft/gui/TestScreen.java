package dev.evvie.waylandcraft.gui;

import org.joml.Matrix3x2fStack;

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
	public void extractRenderState(GuiGraphicsExtractor context, int i, int j, float f) {
		super.extractBlurredBackground(context);
		
		float guiScale = (float) Minecraft.getInstance().getWindow().getGuiScale();
		Matrix3x2fStack poseStack = context.pose();
		poseStack.pushMatrix();
		poseStack.scale(1 / guiScale, 1 / guiScale);
		
		context.fill(0, 0, 100, 100, ARGB.color(255, 0, 125));
//		context.blit(textureView, sampler, x, y, x + w, y + h, 0, 1, 0, 1);
		
		poseStack.popMatrix();
		
		super.extractRenderState(context, i, j, f);
	}
	
	@Override
	public void extractBackground(GuiGraphicsExtractor guiGraphics, int i, int j, float f) {
	}
	
}
